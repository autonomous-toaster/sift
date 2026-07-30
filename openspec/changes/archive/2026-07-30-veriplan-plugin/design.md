## Context

The pi agent runs `veriplan check` through sift's bash proxy. Currently:
1. Default output is human-readable (verbose text)
2. Agent must parse verbose text to extract structured info
3. No token optimization is applied

The rtk plugin already demonstrates the pattern: intercept command, ensure optimal format, compress output, nudge agent. The veriplan plugin follows the same pattern.

## Goals / Non-Goals

**Goals:**
- `veriplan check` output is always JSON (machine-readable)
- Output is compressed to shortest format (compact JSON vs toon)
- Nudge tells agent how to get raw output
- `json_shortest_impl()` correctly accounts for nudge message length

**Non-Goals:**
- No changes to veriplan itself
- No changes to other veriplan subcommands (init, visualize, lsp)
- No changes to the toon-format library

## Decisions

### 1. Plugin architecture

```lua
-- plugins/veriplan.lua
return {
    name = "veriplan",
    priority = 0,
    pattern = "veriplan",

    execute = function(ctx, args, stdin)
        -- Only optimize "check" subcommand
        if args[1] ~= "check" then
            return { status = "passthrough" }
        end

        -- Ensure --json flag
        local has_json = false
        local has_format = false
        for _, arg in ipairs(args) do
            if arg == "--json" then has_json = true end
            if arg == "--format" then has_format = true end
        end

        local new_args = {}
        for _, arg in ipairs(args) do
            table.insert(new_args, arg)
        end
        if not has_json and not has_format then
            table.insert(new_args, "--json")
        end

        local cmd = "veriplan " .. table.concat(new_args, " ")
        local output, stderr, exit_code = sift.exec(ctx, cmd)

        if exit_code ~= 0 then
            return { status = "passthrough" }
        end

        -- json.shortest tries compact JSON, toon, picks shortest
        local optimized = sift.json.shortest(ctx, output, { toon = true })
        return {
            status = "handled",
            output = optimized,
            exit_code = 0,
            streamed = true
        }
    end
}
```

### 2. Nudge overhead fix

Current code in `json_shortest_impl()`:
```rust
let nudge_msg_len = 8 + 20 + nudge_path_str.len();
```

The `20` is a stale estimate. The actual nudge message prefix is `"compressed output. raw: command cat "` (38 chars). Plus `[nudge] ` (8 chars) added by dispatch.

Fix: use the actual format string length:
```rust
let nudge_prefix = "compressed output. raw: command cat ";
let nudge_msg_len = 8 + nudge_prefix.len() + nudge_path_str.len();
```

This ensures the token cost comparison correctly accounts for the nudge overhead when deciding between raw and compressed formats.

### 3. Edge cases

| Input | Behavior |
|-------|----------|
| `veriplan check foo` | Adds `--json`, compresses output |
| `veriplan check foo --json` | Already JSON, compresses output |
| `veriplan check foo --format human` | Overrides to `--json`, compresses |
| `veriplan check foo --verbose` | Verbose JSON, still compresses (json.shortest re-serializes compactly) |
| `veriplan init` | Passthrough (not a check command) |
| `veriplan check` (no args) | Auto-detect changes, multi-change JSON output, still compresses |
| `veriplan check foo --pre-commit` | Pre-commit mode, output format same, compresses |
| veriplan exits non-zero | Passthrough (don't hide errors) |

## Risks / Trade-offs

- **Plugin ordering**: If another plugin also matches `veriplan`, the higher-priority one wins. The veriplan plugin uses priority 0 (same as rtk), which is higher than `__default__` (-500) but lower than specific command plugins.
- **`--json` flag conflict**: If veriplan adds a `--compact` flag in the future, the plugin may need updating. Currently `--json` without `--verbose` is already compact.
- **Multi-change output**: `veriplan check` without a change name auto-detects all changes and produces a different JSON structure (`changes` array). The `json.shortest()` function handles this correctly since it works on any valid JSON.
