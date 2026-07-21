## 1. head.lua

- [x] 1.1 Add `local stat = sift.fs.stat(ctx, path)` after the `sift.fs.read()` call in head.lua
- [x] 1.2 Add `raw_bytes = stat.size` to both unchanged return tables in head.lua
- [x] 1.3 Add `raw_bytes = stat.size` to the handled return table in head.lua

**Files:** `plugins/head.lua`

## 2. tail.lua

- [x] 2.1 Add `local stat = sift.fs.stat(ctx, path)` after the `sift.fs.read()` call in tail.lua
- [x] 2.2 Add `raw_bytes = stat.size` to both unchanged return tables in tail.lua
- [x] 2.3 Add `raw_bytes = stat.size` to the handled return table in tail.lua

**Files:** `plugins/tail.lua`

## 3. sed.lua

- [x] 3.1 Add `local stat = sift.fs.stat(ctx, path)` after the `sift.fs.read()` call in sed.lua
- [x] 3.2 Add `raw_bytes = stat.size` to both unchanged return tables in sed.lua
- [x] 3.3 Add `raw_bytes = stat.size` to the handled return table in sed.lua

**Files:** `plugins/sed.lua`

## 4. openspec.lua

- [x] 4.1 Add `raw_bytes = #(output .. stderr)` to the failure return table in openspec.lua
- [x] 4.2 Add `raw_bytes = #output` to the success return table in openspec.lua, where `output` is the raw openspec output before `sift.json.shortest()` compression

**Files:** `plugins/openspec.lua`

## 5. Verify gain report accuracy

- [x] 5.1 Run `sift -c "head -n 5 Cargo.toml"` twice and confirm the gain report shows non-zero reduction for the head plugin
- [x] 5.2 Run `sift -c "tail -n 5 Cargo.toml"` and confirm the gain report shows non-zero reduction for the tail plugin
- [x] 5.3 Run `sift -c "sed -n '1,5p' Cargo.toml"` and confirm the gain report shows non-zero reduction for the sed plugin
- [x] 5.4 Run `sift -c "openspec list --json"` and confirm the gain report shows non-zero reduction for the openspec plugin
