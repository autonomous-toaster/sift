## 1. sift.str.* builtins (Rust + plugin updates)

- [x] 1.1 Register `sift.str.split_lines()` in `api_reg_io.rs` — Rust function returning Lua table of lines
- [x] 1.2 Register `sift.str.slice_text()` in `api_reg_io.rs` — Rust function for 1-indexed line range extraction
- [x] 1.3 Register `sift.str.is_sensitive()` in `api_reg_io.rs` — Rust function matching against sensitive path patterns
- [x] 1.4 Update head.lua — replace local `split_lines`/`slice_text` with `sift.str.*` calls
- [x] 1.5 Update tail.lua — replace local `split_lines`/`slice_text` with `sift.str.*` calls
- [x] 1.6 Update sed.lua — replace local `split_lines`/`slice_text` with `sift.str.*` calls
- [x] 1.7 Update sift-read.lua — replace local `split_lines`/`slice_text`/`is_sensitive` with `sift.str.*` calls
- [x] 1.8 Update cat.lua — replace local `is_sensitive` with `sift.str.is_sensitive()` call
- [x] 1.9 Remove duplicated `split_lines`, `slice_text`, `is_sensitive` function definitions from all plugins
- [x] 1.10 Add unit tests for `sift.str.*` functions in Rust

## 2. StdinReader Lua userdata (Rust)

- [x] 2.1 Define `StdinReader` struct wrapping `Box<dyn Read + Send>` with mlua `UserData` impl
- [x] 2.2 Implement `readline()` method — reads next line, returns string or nil
- [x] 2.3 Implement `read(n)` method — reads up to N bytes, returns string or nil
- [x] 2.4 Implement `lines()` method — returns Lua iterator function
- [x] 2.5 Implement `tostring()` method — reads entire stream into a string (backward compat)
- [x] 2.6 Add unit tests for StdinReader in Rust

## 3. Streaming stdin dispatch (Rust)

- [x] 3.1 Update `dispatch()` signature to accept `Option<StdinReader>` alongside `Option<&str>`
- [x] 3.2 Update `dispatch_full()` — for `< file`, open file and create `StdinReader` wrapping `BufReader<File>`
- [x] 3.3 Update pipeline handler — create `StdinReader` wrapping `Cursor<String>` from collected output
- [x] 3.4 Update existing plugins to use `stdin:readline()`, `stdin:read()`, or `stdin:lines()` instead of raw string
- [x] 3.5 Add integration tests for streaming stdin with `< file` and piped input

## 4. Extension notifications and session reset (TypeScript + Lua)

- [x] 4.1 Update reset.lua — add cache hit detection, return `"(cleared)"` or `"(nothing to clear)"`
- [x] 4.2 Update sift.ts — replace silent `catch {}` with `ctx.ui.notify()` on success and error
- [x] 4.3 Update sift.ts — remove `session_shutdown`, `session_tree`, `session_fork`, `session_switch` reset handlers
- [x] 4.4 Update sift.ts — keep only `session_compact` reset handler
