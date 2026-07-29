## Context

sift plugins are Lua scripts in `plugins/` that match command patterns and can intercept, transform, or delegate execution. The `sift.exec()` function runs a command in bash and returns `(stdout, stderr, exit_code)`. Currently it does not accept stdin input — the `stdin` parameter in the underlying Rust `exec_command()` is hardcoded to `None`.

scred is an external binary at an unknown path (resolved via `PATH`). It reads stdin, detects secrets using pattern matching (FastPrefix, StructuredFormat, RegexBased), and outputs redacted text. It supports `--env-mode` for `KEY=VALUE` format and `--text-mode` for free-form text.

## Goals / Non-Goals

**Goals:**
- New `plugins/scred.lua` that matches `echo`, `env`, `printenv`
- Plugin executes the command, captures output, pipes through `scred`, returns redacted result
- `sift.exec()` accepts `{stdin = "..."}` option to pass data as stdin to the subprocess
- Graceful fallback if `scred` is not installed or fails

**Non-goals:**
- No changes to scred itself
- No pre-execution env scrubbing
- No redaction of other command types

## Architecture

```
Agent: echo $AWS_SECRET_ACCESS_KEY
  │
  ▼
dispatch() → pattern match "echo" → scred.lua
  │
  ▼
scred.lua.execute(ctx, args, stdin)
  │
  ├── 1. Reconstruct cmd from ctx.command + args
  │
  ├── 2. out, _, code = sift.exec(ctx, cmd)
  │      └── exec_command() → bash -c "echo $AWS_KEY"
  │
  ├── 3. if code == 0 and #out > 0:
  │      redacted, _, _ = sift.exec(ctx, "scred", {stdin = out})
  │      └── exec_command() → bash -c "scred" with stdin=out
  │
  └── 4. return { status = "handled", output = redacted, exit_code = 0 }
```

## Detailed Design

### 1. Rust: stdin option for sift.exec()

**File:** `sift-core/src/lua/api_reg_cache.rs`

Extract `stdin` from the opts table and pass it to `exec_command()`:

```rust
let stdin = opts
    .as_ref()
    .and_then(|t| t.get::<String>("stdin").ok());

let (stdout, stderr, exit_code) = exec_command(
    &cmd, &session_id, cmd_count,
    transform, silent, merge_stderr,
    stdin.as_deref(),  // was: None
)?;
```

`exec_command()` already accepts `Option<String>` for stdin and writes it to the child's stdin pipe. No changes needed there.

### 2. Plugin: plugins/scred.lua

```lua
return {
    name = "scred",
    priority = 0,
    pattern = { "echo", "env", "printenv" },
    append_prompt = "Output from echo, env, and printenv is redacted by scred. " ..
        "Use 'command' prefix to bypass.",

    execute = function(ctx, args, stdin)
        -- Reconstruct the original command
        local parts = { ctx.command }
        for i = 1, #args do
            parts[#parts + 1] = sift.str.shell_quote(ctx, args[i])
        end
        local cmd = table.concat(parts, " ")

        -- Execute the command, capture output
        local out, stderr, code = sift.exec(ctx, cmd)
        if code ~= 0 or #out == 0 then
            return { status = "passthrough" }
        end

        -- Pipe output through scred
        local redacted, _, scred_code = sift.exec(ctx, "scred", { stdin = out })
        if scred_code == 0 then
            return {
                status = "handled",
                output = redacted,
                exit_code = 0
            }
        end

        -- scred failed — return original output
        return {
            status = "handled",
            output = out,
            exit_code = 0
        }
    end
}
```

### 3. Edge case handling

| Case | Behavior |
|---|---|
| `scred` not installed | `sift.exec("scred")` returns non-zero → passthrough original output |
| `scred` fails on input | Same — return original output |
| Empty output (`echo` with no args) | Skip scred, return empty |
| `echo -n` | Output has no newline — scred handles it |
| `echo 'literal $VAR'` | Shell doesn't expand in single quotes — output is literal, scred passes through |
| `echo $UNDEFINED` | Empty output — skip scred |
| `env` with many vars | scred processes line-by-line, handles large input |
| Binary/non-UTF8 in output | scred uses `String::from_utf8_lossy` — safe |

### 4. Testing

- Unit test for `sift.exec()` with stdin option
- Integration test: run `echo hello` through sift with scred plugin, verify output is redacted
- Integration test: scred not in PATH, verify passthrough
- Integration test: `env` with a test variable, verify redaction

## Alternatives Considered

**Append `| scred` to command string:** Simpler but fails if command already has pipes or redirects. Also requires `scred` to be in PATH at command time rather than plugin time.

**Temp file approach:** Writing output to disk and reading back is fragile (permissions, cleanup, race conditions). Rejected.

**Inline scred as Rust extension:** Would require embedding scred's detection logic in sift. Much higher complexity. scred as external binary is simpler and keeps concerns separated.
