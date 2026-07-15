## Context

`exec_command()` is the single choke point for all command execution — passthrough, plugin `sift.exec()`, and pipeline segments all go through it. Currently it uses `std::process::Command::output()` which buffers all output until the process exits. Changing it to stream makes every path stream automatically.

## Goals / Non-Goals

**Goals:**
- All command output streams to stdout/stderr in real-time
- `sift.exec()` accepts optional transform callback for chunk-by-chunk transformation
- All existing plugins work unchanged
- Function signatures and return types unchanged

**Non-Goals:**
- No changes to plugin API beyond optional transform parameter
- No changes to agent_mode() or repl_mode()
- No streaming JSON parser for shortest (raw-stream-then-optimize-return is sufficient)

## Decisions

### D1 — Thread-based chunk reading

```rust
fn exec_command(cmd, session_id, cmd_count, transform?) -> (String, String, i32) {
    let mut child = Command::new(bash).arg("-c").arg(cmd).spawn()?;
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    // Spawn reader threads for stdout and stderr
    let stdout_buf = Arc::new(Mutex::new(String::new()));
    let stderr_buf = Arc::new(Mutex::new(String::new()));

    let (stdout_handle, stderr_handle) = {
        // stdout thread: read chunks, transform, write to stdout, collect
        // stderr thread: read chunks, write to stderr, collect
    };

    child.wait()?;
    stdout_handle.join()?;
    stderr_handle.join()?;

    Ok((stdout_buf, stderr_buf, exit_code))
}
```

**Rationale**: Threads are simple and reliable for this pattern. Each stream gets its own thread. The main thread waits for the child process and then joins the reader threads.

### D2 — Transform callback in sift.exec

The Lua `sift.exec()` binding accepts an optional third parameter:

```lua
sift.exec(ctx, "cmd")                          -- raw streaming
sift.exec(ctx, "cmd", { transform = fn })      -- transformed streaming
```

The transform function receives a string chunk and returns a (possibly modified) string. It's called for each stdout chunk before writing and collecting.

**Rationale**: Minimal API surface. Optional parameter doesn't break existing plugins. The transform is applied per-chunk, so it works for streaming transformers (uppercase, grep-like filters) but not for buffered transformers (shortest needs full output).

## Risks / Trade-offs

- **Thread overhead**: Two threads per command execution. Negligible for a shell proxy — process spawn overhead dominates.
- **Binary output**: `String::from_utf8_lossy` handles non-UTF-8 gracefully.
- **Transform on stderr**: Transform only applies to stdout. Stderr always streams raw. This keeps the API simple and matches the common use case (transforming command output, not errors).
