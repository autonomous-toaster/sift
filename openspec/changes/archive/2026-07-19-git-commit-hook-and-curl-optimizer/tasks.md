## 1. git-commit-hook plugin

- [ ] 1.1 Create `plugins/git-commit.lua` — pattern `"git commit"`, detect `-n`/`--no-verify`, return error + nudge
- [ ] 1.2 Handle false positives: skip args that are values to flags (`-m`, `-F`, `-C`, `-t`, etc.)
- [ ] 1.3 Add plugin tests for git-commit scenarios

## 2. curl-json-optimizer plugin

- [ ] 2.1 Create `plugins/curl.lua` — pattern `"curl"`, detect `-v`/`--verbose`, add `-w "\n%{content_type}"` when absent
- [ ] 2.2 Parse stdout: split body from content type via last newline
- [ ] 2.3 JSON detection + compression: `sift.json.shortest()` + `sift.store()` raw + nudge
- [ ] 2.4 Non-JSON passthrough: return body as-is, propagate exit code
- [ ] 2.5 Add plugin tests for curl scenarios (httpbin.org)
