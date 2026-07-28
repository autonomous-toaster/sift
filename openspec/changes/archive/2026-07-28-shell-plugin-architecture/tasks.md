## Phase 1: Narrow rtk patterns

- [x] 1.1 In `plugins/rtk.lua`, change `pattern = "*"` to a list of commands rtk handles: `ls`, `tree`, `read`, `git`, `gh`, `glab`, `aws`, `psql`, `pnpm`, `err`, `test`, `json`, `deps`, `env`, `find`, `diff`, `log`, `dotnet`, `docker`, `kubectl`, `summary`, `init`, `wget`, `wc`, `gain`, `cc-economics`, `config`, `jest`, `vitest`, `prisma`, `tsc`, `next`, `lint`, `smart`
- [x] 1.2 Verify: `sed -i 's/foo/bar/' file` no longer matches rtk (goes to shell plugin)
- [x] 1.3 Verify: `git status` still matches rtk

## Phase 2: Add `ctx.original_cmd` to plugin context

- [x] 2.1 In `sift-core/src/lua/api.rs`, add `original_cmd` field to the context table in `dispatch()`, using the `original_cmd` parameter already threaded through
- [x] 2.2 Verify: plugins can access `ctx.original_cmd` in their `execute()` function

## Phase 3: Remove `__default__`/`*` bypass and update shell plugin

- [x] 3.1 In `sift-core/src/lua/api.rs`, remove the bypass block in `dispatch()` that checks for `__default__` or `*` patterns and calls `exec_command()` directly
- [x] 3.2 In `sift/plugins/bash.lua`, update `execute()` to use `ctx.original_cmd` instead of shell-quoting args, calling `sift.exec(ctx, ctx.original_cmd)`
- [x] 3.3 In `sift-core/src/lua/api.rs`, update the pipeline fallback in `try_pipeline()` to dispatch through `self.dispatch()` with the full pipeline string instead of calling `exec_command()` directly
- [x] 3.4 Verify: `FILE=value echo test` preserves shell semantics (variable expansion works)
- [x] 3.5 Verify: `ls -la` runs correctly through the shell plugin
- [x] 3.6 Verify: `echo 1 | head` runs through the shell plugin (pipeline fallback)
- [x] 3.7 Verify: all existing tests pass
