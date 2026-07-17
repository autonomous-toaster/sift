//! Streaming stdin reader for Lua plugins.
//!
//! Provides `StdinReader` — a Lua userdata that wraps `Box<dyn Read + Send>`.
//! Plugins read incrementally via `readline()`, `read(n)`, or `lines()`.
//! Backed by `BufReader<File>` for `< file` redirects or `Cursor<String>` for piped input.

use std::io::{BufRead, BufReader, Cursor, Read};
use std::sync::{Arc, Mutex};

use mlua::{UserData, UserDataMethods, Value};

type InnerReader = Arc<Mutex<Inner>>;

enum Inner {
    File(BufReader<std::fs::File>),
    String(Cursor<String>),
}

/// A streaming stdin reader for Lua plugins.
///
/// Wraps either a `BufReader<File>` (for `< file` redirects) or a
/// `Cursor<String>` (for piped/string input). Uses interior mutability
/// via `Arc<Mutex<...>>` so the reader can be shared across Lua iterator calls.
#[derive(Clone)]
pub struct StdinReader {
    inner: InnerReader,
}

impl StdinReader {
    /// Create a file-backed reader.
    pub fn from_file(file: std::fs::File) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner::File(BufReader::new(file)))),
        }
    }

    /// Create a string-backed reader.
    pub fn from_string(s: String) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner::String(Cursor::new(s)))),
        }
    }

    /// Read the entire stream into a string (for backward compat).
    pub fn read_to_string(&self) -> std::io::Result<String> {
        let mut guard = self.inner.lock().unwrap();
        match &mut *guard {
            Inner::File(r) => {
                let mut s = String::new();
                r.read_to_string(&mut s)?;
                Ok(s)
            }
            Inner::String(r) => {
                let mut s = String::new();
                r.read_to_string(&mut s)?;
                Ok(s)
            }
        }
    }

    fn readline_inner(guard: &mut Inner) -> std::io::Result<Option<String>> {
        let mut buf = String::new();
        let n = match guard {
            Inner::File(r) => r.read_line(&mut buf)?,
            Inner::String(r) => r.read_line(&mut buf)?,
        };
        if n == 0 {
            return Ok(None);
        }
        // Strip trailing newline
        if buf.ends_with('\n') {
            buf.pop();
        }
        Ok(Some(buf))
    }
}

impl UserData for StdinReader {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        // __tostring metamethod for backward compat (tostring(stdin))
        methods.add_meta_method("__tostring", |_lua, reader, ()| {
            let mut guard = reader.inner.lock().unwrap();
            match &mut *guard {
                Inner::File(r) => {
                    let mut s = String::new();
                    r.read_to_string(&mut s).map_err(|e| mlua::Error::external(format!("read: {e}")))?;
                    Ok(s)
                }
                Inner::String(r) => {
                    let mut s = String::new();
                    r.read_to_string(&mut s).map_err(|e| mlua::Error::external(format!("read: {e}")))?;
                    Ok(s)
                }
            }
        });

        // stdin:readline() -> string | nil
        methods.add_method("readline", |lua, reader, ()| {
            let mut guard = reader.inner.lock().unwrap();
            match Self::readline_inner(&mut guard) {
                Ok(Some(line)) => Ok(Value::String(lua.create_string(&line)?)),
                Ok(None) => Ok(Value::Nil),
                Err(e) => Err(mlua::Error::external(format!("readline: {e}"))),
            }
        });

        // stdin:read(n) -> string | nil
        methods.add_method("read", |lua, reader, n: u64| {
            let n = n as usize;
            let mut buf = vec![0u8; n];
            let mut guard = reader.inner.lock().unwrap();
            let bytes_read = match &mut *guard {
                Inner::File(r) => r.read(&mut buf)?,
                Inner::String(r) => r.read(&mut buf)?,
            };
            if bytes_read == 0 {
                return Ok(Value::Nil);
            }
            buf.truncate(bytes_read);
            let s = String::from_utf8_lossy(&buf).to_string();
            Ok(Value::String(lua.create_string(&s)?))
        });

        // stdin:lines() -> iterator function
        methods.add_method("lines", |lua, reader, ()| {
            let inner = Arc::clone(&reader.inner);
            let func = lua.create_function(move |lua, ()| {
                let mut guard = inner.lock().unwrap();
                match StdinReader::readline_inner(&mut guard) {
                    Ok(Some(line)) => Ok(Value::String(lua.create_string(&line)?)),
                    Ok(None) => Ok(Value::Nil),
                    Err(e) => Err(mlua::Error::external(format!("readline: {e}"))),
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
        let result: mlua::Result<()> = lua.load(
            "local s = tostring(r)\nassert(s == 'hello\\nworld\\n')\n"
        ).exec();
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
