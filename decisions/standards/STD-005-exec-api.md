# STD-005 · sift.exec() API and Environment Contract

## Signature

```lua
-- Execute a shell command and capture output.
-- Returns (stdout, stderr, exit_code).
-- stdout and stderr are strings. exit_code is an integer (0 = success).
local stdout, stderr, exit_code = sift.exec(cmd)
```

## Parameters

| Param | Type | Description |
|---|---|---|
| `cmd` | string | Shell command to execute. Passed to `bash -c`. |

## Return values

| # | Name | Type | Description |
|---|---|---|---|
| 1 | `stdout` | string | Standard output of the command. |
| 2 | `stderr` | string | Standard error of the command. |
| 3 | `exit_code` | integer | Exit code of the command (0 = success). |

## Implementation

`sift.exec()` uses `std::process::Command` with piped stdout and stderr. No PTY is involved.

```rust
fn exec_command(cmd: &str) -> Result<(String, String, i32)> {
    let output = std::process::Command::new(&find_real_bash())
        .arg("-c")
        .arg(cmd)
        .envs(SIFT_EXEC_ENV)  // see environment contract below
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(1);

    Ok((stdout, stderr, exit_code))
}
```

## Environment contract

Every process spawned by `sift.exec()` receives these environment variables, set via `std::process::Command::env()`:

| Variable | Value | Reason |
|---|---|---|
| `PAGER` | `cat` | Prevent pager blocking (git diff, less, etc.) |
| `TERM` | `dumb` | Prevent ANSI color codes, disable TTY-specific features |
| `EDITOR` | `true` | Prevent editor blocking (git rebase --continue, etc.) |
| `GIT_EDITOR` | `true` | Belt-and-suspenders for git |
| `GIT_PAGER` | `cat` | Belt-and-suspenders for git |

The child process inherits all other environment variables from the parent. These five are overridden to ensure non-interactive, plain-text output suitable for AI agent consumption.

## Passthrough execution

When a plugin returns `status = "passthrough"`, the command is executed using the same `exec_command()` function. Passthrough is not a separate code path — it uses the same pipe-based subprocess with the same environment contract.

## Error handling

- If the command cannot be spawned (e.g., bash not found), `sift.exec()` returns an error to the Lua plugin.
- If the command exits with a non-zero code, `exit_code` reflects it. The plugin is responsible for handling non-zero exit codes.
- stdout and stderr are always returned as strings, even if empty. They are never nil.
