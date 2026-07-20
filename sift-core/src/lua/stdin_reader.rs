//! Streaming stdin reader for Lua plugins.
//!
//! Provides `StdinReader` — a Lua userdata that wraps pre-loaded text content.
//! Plugins read incrementally via `readline()`, `read(n)`, or `lines()`.
//! Uses `Cell<usize>` for position tracking instead of `Arc<Mutex<...>>`
//! since readers are only used within a single thread.

use std::cell::Cell;
use std::io::Read;
use std::sync::{Arc, Mutex};

use mlua::{UserData, UserDataMethods, Value};

/// A streaming stdin reader for Lua plugins.
///
/// Pre-loads all content during construction. Uses `Cell<usize>` for
/// position tracking — no `Arc<Mutex<...>>` overhead since readers
/// are only used within a single thread.
pub struct StdinReader {
    /// Pre-loaded content bytes.
    data: Vec<u8>,
    /// Current read position (byte offset).
    pos: Cell<usize>,
}

impl StdinReader {
    /// Create a file-backed reader.
    pub fn from_file(mut file: std::fs::File) -> Self {
        let mut data = Vec::new();
        let _ = file.read_to_end(&mut data);
        Self {
            data,
            pos: Cell::new(0),
        }
    }

    /// Create a string-backed reader.
    pub fn from_string(s: String) -> Self {
        Self {
            data: s.into_bytes(),
            pos: Cell::new(0),
        }
    }

    /// Read the entire stream into a string (for backward compat).
    pub fn read_to_string(&self) -> std::io::Result<String> {
        Ok(String::from_utf8_lossy(&self.data).to_string())
    }

    /// Read the next line (without trailing newline).
    fn readline_impl(&self) -> Option<String> {
        let pos = self.pos.get();
        if pos >= self.data.len() {
            return None;
        }
        // Find next newline
        let remaining = &self.data[pos..];
        let newline_pos = remaining.iter().position(|&b| b == b'\n');
        match newline_pos {
            Some(nl) => {
                let line = String::from_utf8_lossy(&remaining[..nl]).to_string();
                self.pos.set(pos + nl + 1); // skip past newline
                Some(line)
            }
            None => {
                let line = String::from_utf8_lossy(remaining).to_string();
                self.pos.set(self.data.len());
                Some(line)
            }
        }
    }
}

impl UserData for StdinReader {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        // __tostring metamethod for backward compat (tostring(stdin))
        methods.add_meta_method("__tostring", |_lua, reader, ()| {
            Ok(String::from_utf8_lossy(&reader.data).to_string())
        });

        // stdin:readline() -> string | nil
        methods.add_method("readline", |lua, reader, ()| match reader.readline_impl() {
            Some(line) => Ok(Value::String(lua.create_string(&line)?)),
            None => Ok(Value::Nil),
        });

        // stdin:read(n) -> string | nil
        methods.add_method("read", |lua, reader, n: u64| {
            #[allow(clippy::cast_possible_truncation)]
            let n = n as usize;
            let pos = reader.pos.get();
            if pos >= reader.data.len() {
                return Ok(Value::Nil);
            }
            let end = (pos + n).min(reader.data.len());
            let chunk = &reader.data[pos..end];
            reader.pos.set(end);
            let s = String::from_utf8_lossy(chunk).to_string();
            Ok(Value::String(lua.create_string(&s)?))
        });

        // stdin:lines() -> iterator function
        methods.add_method("lines", |lua, reader, ()| {
            let data = Arc::new(reader.data.clone());
            let pos = Arc::new(Mutex::new(reader.pos.get()));
            let func = lua.create_function(move |lua, ()| {
                let current = {
                    let p = pos.lock().unwrap_or_else(|e| e.into_inner());
                    *p
                };
                if current >= data.len() {
                    return Ok(Value::Nil);
                }
                let remaining = &data[current..];
                let newline_pos = remaining.iter().position(|&b| b == b'\n');
                match newline_pos {
                    Some(nl) => {
                        let line = String::from_utf8_lossy(&remaining[..nl]).to_string();
                        if let Ok(mut p) = pos.lock() {
                            *p = current + nl + 1;
                        }
                        Ok(Value::String(lua.create_string(&line)?))
                    }
                    None => {
                        let line = String::from_utf8_lossy(remaining).to_string();
                        if let Ok(mut p) = pos.lock() {
                            *p = data.len();
                        }
                        Ok(Value::String(lua.create_string(&line)?))
                    }
                }
            })?;
            Ok(func)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::Lua;

    #[test]
    fn test_readline_string() {
        let lua = Lua::new();
        let reader = StdinReader::from_string("hello\nworld\n".to_string());
        let globals = lua.globals();
        globals.set("r", reader).unwrap();
        let result: mlua::Result<()> = lua.load(
            "local line1 = r:readline()\nassert(line1 == 'hello')\nlocal line2 = r:readline()\nassert(line2 == 'world')\nlocal line3 = r:readline()\nassert(line3 == nil)\n"
        ).exec();
        assert!(result.is_ok(), "{result:?}");
    }

    #[test]
    fn test_read_bytes() {
        let lua = Lua::new();
        let reader = StdinReader::from_string("hello world".to_string());
        let globals = lua.globals();
        globals.set("r", reader).unwrap();
        let result: mlua::Result<()> = lua.load(
            "local chunk = r:read(5)\nassert(chunk == 'hello')\nlocal chunk2 = r:read(100)\nassert(chunk2 == ' world')\nlocal chunk3 = r:read(5)\nassert(chunk3 == nil)\n"
        ).exec();
        assert!(result.is_ok(), "{result:?}");
    }

    #[test]
    fn test_lines_iterator() {
        let lua = Lua::new();
        let reader = StdinReader::from_string("a\nb\nc\n".to_string());
        let globals = lua.globals();
        globals.set("r", reader).unwrap();
        let result: mlua::Result<()> = lua.load(
            "local count = 0\nfor line in r:lines() do\n    count = count + 1\nend\nassert(count == 3)\n"
        ).exec();
        assert!(result.is_ok(), "{result:?}");
    }

    #[test]
    fn test_tostring() {
        let lua = Lua::new();
        let reader = StdinReader::from_string("hello\nworld\n".to_string());
        let globals = lua.globals();
        globals.set("r", reader).unwrap();
        let result: mlua::Result<()> = lua
            .load("local s = tostring(r)\nassert(s == 'hello\\nworld\\n')\n")
            .exec();
        assert!(result.is_ok(), "{result:?}");
    }

    #[test]
    fn test_file_reader() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "line1").unwrap();
        writeln!(f, "line2").unwrap();
        drop(f);

        let file = std::fs::File::open(&path).unwrap();
        let reader = StdinReader::from_file(file);
        let lua = Lua::new();
        let globals = lua.globals();
        globals.set("r", reader).unwrap();
        let result: mlua::Result<()> = lua.load(
            "local line1 = r:readline()\nassert(line1 == 'line1')\nlocal line2 = r:readline()\nassert(line2 == 'line2')\nlocal line3 = r:readline()\nassert(line3 == nil)\n"
        ).exec();
        assert!(result.is_ok(), "{result:?}");
    }
}
