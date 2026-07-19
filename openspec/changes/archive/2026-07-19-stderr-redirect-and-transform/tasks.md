## 1. Core dispatch changes

- [x] 1.1 Add `parse_fd_redirects()` — detect `N>&M` patterns in args, return merge map
- [x] 1.2 Wire fd redirect detection into `dispatch_with_redirect` — strip matched args, set `merge_stderr` flag
- [x] 1.3 Add `merge_stderr` param to `exec_command` — keep stdout/stderr separate, apply transform to stdout only, append stderr
- [x] 1.4 Add `merge_stderr` option to `sift.exec` — pass from Lua options to `exec_command`
- [x] 1.5 Pipeline deferral — skip redirect handling when `|` detected in command

## 2. Plugin fix

- [x] 2.1 Fix `bash.lua` to return `output` in result table — use `sift.exec` return value

## 3. Tests

- [x] 3.1 Update `exec_command` tests for new `merge_stderr` param
- [x] 3.2 Update plugin tests for bash.lua returning output