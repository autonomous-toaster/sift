## 1. Project scaffold and parser

- [x] 1.1 Initialize Cargo workspace with brush-parser, sqlx, os_pipe, sha2, anyhow, clap dependencies. Configure lints per STD-001. Create directory structure per STD-002.
- [x] 1.2 Implement `parser.rs` — thin wrapper over brush-parser that parses a single line of shell input into a `Program` AST. Handle parse errors gracefully (return error, don't panic). Add unit tests for parsing common patterns: simple command, pipeline, redirect, &&/||, subshell, heredoc, env var.
- [x] 1.3 Implement `plugin.rs` — `Plugin` trait, `PluginResult` enum, `PluginRegistry` with `register()` and `find()` methods. Add unit tests for registry: register plugin, find by name, return None for unknown command.

## 2. Dispatcher core

- [x] 2.1 Implement `dispatcher.rs` — `dispatch()` function that takes a parsed `Program`, walks the AST, and for each `SimpleCommand` decides: plugin or exec? Implement the interception rule: only invoke plugin if stdout goes to PTY (not piped, not redirected). For non-interceptable commands, return `Passthrough`. Add unit tests for dispatch decisions: simple command → plugin, piped command → passthrough, redirected command → passthrough.
- [x] 2.2 Implement pipeline handling in dispatcher. If a pipeline has no plugin-invoked commands, delegate to `/bin/bash -c`. If at least one command is a plugin, set up `os_pipe` pipes manually between commands. Handle the last command's output going to stdout. Add unit tests for pipeline execution: all-real → bash delegation, mixed → manual pipes, all-plugin → manual pipes.
- [x] 2.3 Implement `builtins.rs` — `cd` (with POSIX semantics: `cd`, `cd -`, `cd ~`, `cd /path`, track cwd and OLDPWD), `export` (set env var in session), `unset` (remove env var), `exit` (exit shell). Add unit tests for each builtin: cd changes cwd, cd - goes back, export sets var, unset removes var, exit terminates.

## 3. Session store

- [x] 3.1 Implement `session.rs` — `Session` struct with `cmd_count`, `cwd`, `env` fields. `SessionStore` struct wrapping sqlx SQLite connection. Implement `open()`, `get_file_cache()`, `upsert_file_cache()`, `get_conversation()`, `record_conversation()`, `increment_re_requested()`. Create the SQLite schema (file_cache, conversation_cache tables) on first open. Add unit tests for each store method.
- [x] 3.2 Wire session into the REPL loop in `main.rs`. Check `AI_SESSION` env var: if set, open session store; if not, run without session (stateless fallback). Increment `cmd_count` on each command. Add integration test: session store is created when AI_SESSION is set, not created when absent.

## 4. Cat plugin

- [x] 4.1 Implement `plugins/cat.rs` — `CatPlugin` struct implementing `Plugin`. v1 scope: only intercept `cat <file>` with no flags. Parse args: if any flags or multiple files, return `Passthrough`. Read file, compute SHA256, check conversation cache. If cache hit and fresh (commands_since < 50), return `Unchanged` with informative message. If cache miss or stale, return `Handled` with full content. Update cache. Add unit tests: cat single file, cat with flags → passthrough, cat nonexistent file → error, repeated cat → unchanged marker.
- [x] 4.2 Register CatPlugin in `main.rs`. Wire the REPL loop: read line, parse, dispatch, write output. Handle EOF (exit). Handle errors gracefully (print to stderr, continue). Add integration test: run baish with a command, verify output matches expected.

## 5. Startup and .bashrc sourcing

- [x] 5.1 Implement `.bashrc` sourcing in `main.rs` startup. On startup, check if `~/.bashrc` exists. If yes, read it, parse with brush-parser, dispatch each complete command through baish's pipeline. Handle errors gracefully (if .bashrc has syntax errors, log and continue). Add integration test: create a .bashrc with aliases and env vars, start baish, verify aliases and env vars are available.
- [x] 5.2 Implement `source` builtin in `builtins.rs`. Read the file, parse with brush-parser, dispatch each complete command. State changes (cwd, env) persist in session. Add unit tests: source a file with cd and export, verify state changes persist.

## 6. Git plugin (v1)

- [ ] 6.1 Implement `plugins/git.rs` — `GitPlugin` struct. v1 scope: intercept `git status` and `git diff`. For `git status`: exec `git status --porcelain=v2`, parse output, render traditional format. Cache fingerprint (HEAD + index + worktree hash). If fingerprint unchanged, emit `[baish] git status unchanged since last check`. For `git diff`: exec `git diff`, cache fingerprint. Add unit tests: git status output matches expected format, repeated git status → unchanged marker.
- [ ] 6.2 Register GitPlugin in `main.rs`. Add integration test: run git status in a repo, verify output format.

## 7. Cargo plugin (v1)

- [ ] 7.1 Implement `plugins/cargo.rs` — `CargoPlugin` struct. v1 scope: intercept `cargo check` and `cargo test`. For `cargo check`: exec `cargo check --message-format=json`, filter to error/warning messages only, render compact output. Cache fingerprint (Cargo.lock + src hash). For `cargo test`: exec `cargo test --message-format=json`, filter to test results only, render compact output. Add unit tests: cargo check output is compact, repeated cargo check → unchanged marker.
- [ ] 7.2 Register CargoPlugin in `main.rs`. Add integration test: run cargo check in a Rust project, verify compact output.
