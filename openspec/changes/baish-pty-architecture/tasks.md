## 1. Multi-crate workspace and core traits

- [x] 1.1 Restructure project into Cargo workspace with three crates: `baish-core` (library), `baish-filters` (library), `baish` (binary). Move existing code into appropriate crates. Configure lints per STD-001.
- [x] 1.2 Define `PluginContext` struct, `PluginResult` enum, `Plugin` trait in `baish-core`. Update `PluginRegistry` with priority-based registration and longest-prefix subcommand matching. Add unit tests for registry: priority ordering, subcommand matching, override semantics.
- [x] 1.3 Define `StreamFilter` trait with `feed_line()`, `flush()`, `on_exit()` methods in `baish-filters`. Implement `PassthroughFilter` (pass all through). Add unit tests for filter pipeline: line-by-line processing, exit summary.

## 2. PTY management

- [x] 2.1 Implement PTY creation in `baish` binary using `portable-pty` crate. Spawn real bash as child process. Set up process group for signal forwarding. Add integration test: PTY is created, bash is running, can send command and read output.
- [x] 2.2 Implement PTY read loop: async read from PTY master, split into lines, feed through filter pipeline, write filtered output to stdout. Handle partial lines at chunk boundaries. Add unit tests for line splitting: chunk with partial line, chunk with multiple lines.
- [x] 2.3 Implement signal handling: forward SIGINT/SIGTERM to child process group, handle SIGWINCH for terminal resize. Add integration test: signal is forwarded, child exits cleanly.

## 3. Command classification and dispatch

- [x] 3.1 Implement `CommandKind` enum and `classify()` function in `baish-core`. Use brush-parser to parse input, extract command name and arguments, determine if piped or compound. Add unit tests for classification: simple command, piped, compound with &&, redirect.
- [x] 3.2 Implement decision engine in `baish` binary: for simple FileRead with cache hit, skip PTY and emit cached marker. For compound or cache miss, send to PTY. Add integration test: simple cat with cache hit skips PTY, compound command goes through PTY.
- [x] 3.3 Implement pre-compute hash logic: for FileRead commands, hash the file before execution. Pass expected hash to filter for early output dedup. Add unit tests: hash computation, cache lookup, stale detection.
- [x] 3.4 Add `rewrite()` method to `Plugin` trait with default no-op implementation. Update dispatcher to call `rewrite()` before sending to PTY. Add unit tests: plugin rewrites command, plugin returns None (no rewrite).
- [x] 3.5 Implement full output storage: write raw PTY output to temp files at `/tmp/baish/<session_id>/<timestamp>_<command>.log`. Include full output path in filter output when output is truncated or summarized. Add unit tests: file is created, path is correct, content matches.
- [x] 3.6 Implement temp file cleanup: on session end, remove session directory. Add configurable max disk usage. Add unit tests: cleanup on exit, max size enforcement.

## 4. Agent mode and human mode

- [x] 4.1 Implement agent mode: parse `-c` argument, execute command, stream filtered output to stdout, exit with child's exit code. Add integration test: `bash -c 'echo hello'` outputs "hello" and exits 0.
- [x] 4.2 Implement human mode: detect TTY stdin, start ratatui TUI with two panes (agent view, raw output). Read input from user, execute through same pipeline as agent mode. Add integration test: human mode starts, shows prompt, executes command.
- [x] 4.3 Implement PS1 prompt for human mode: show `baish$ ` with hostname and cwd. Let real bash's PS1 through to the raw output pane. Add unit test: prompt format matches expected.

## 5. Output filters

- [x] 5.1 Implement `CatFilter` in `baish-filters`: buffer output, compute hash, compare with cache, emit "unchanged" marker on match. Support pre-computed expected hash for early dedup. Add unit tests: first read emits full, repeated read emits marker, file change emits full.
- [x] 5.2 Implement `CargoTestFilter` in `baish-filters`: suppress individual test lines, count passed/failed, emit summary on exit. On success: "✓ N tests passed". On failure: show failed test names and locations. Add unit tests: all pass emits summary, some fail shows failures.
- [x] 5.3 Implement `GitStatusFilter` in `baish-filters`: fingerprint HEAD+index+worktree, emit "working tree clean" on unchanged. Add unit tests: clean tree emits marker, dirty tree shows diff.

## 6. Session store

- [x] 6.1 Move session store to `~/.baish/sessions.db` (single DB). Update `SessionStore::open()` to use home directory path. Add unit tests: DB is created at correct path, multiple sessions share same DB.
- [x] 6.2 Update `PluginContext` to include `session_id` for cache access. Plugins access cache through context, not directly. Add unit tests: context is passed correctly, cache operations work through context.

## 7. Cat plugin with Lua override example

- [x] 7.1 Refactor `CatPlugin` to use `PluginContext` instead of `Session`. Register with priority -100. Add unit tests: cat with context works, priority resolution works.
- [x] 7.2 Create example Lua cat plugin at `docs/examples/cat.lua`. Document the plugin API (PluginContext fields, PluginResult variants, cache functions). Add integration test: Lua plugin overrides built-in cat.
