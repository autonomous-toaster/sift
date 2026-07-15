## 1. Wildcard pattern support

- [x] 1.1 Add `"*"` wildcard matching in `find_plugin()` — a plugin with pattern `"*"` matches any command candidate.

## 2. rtk plugin update

- [x] 2.1 Change rtk.lua pattern from hardcoded list to `"*"`.
- [x] 2.2 Simplify rtk.lua execute logic: try `rtk <command>`, passthrough on non-zero exit.

## 3. Cleanup

- [x] 3.1 Verify `just ci` passes with all changes.
