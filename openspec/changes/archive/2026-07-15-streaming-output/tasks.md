## 1. Streaming exec_command

- [x] 1.1 Rewrite `exec_command()` to spawn process, read chunks in threads, write to stdout/stderr, collect for return.

## 2. Transform callback

- [x] 2.1 Update `sift.exec()` Lua binding to accept optional `{ transform = fn }` parameter.

## 3. Cleanup

- [x] 3.1 Verify `just ci` passes with all changes.
