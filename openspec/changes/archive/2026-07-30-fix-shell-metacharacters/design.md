## Context

`dispatch_full()` in `sift-core/src/lua/api.rs` splits the command string using `shlex::split()` to get the command name and arguments for plugin matching. `shlex::split()` only handles `"`, `'`, `\`, and whitespace — it does not recognize shell metacharacters like `;`, `&&`, `||` as separators.

The existing `split_pipeline()` function already has quote-aware logic for detecting `|` outside quotes. This same state machine can be reused for `;`, `&&`, `||`.

## Goals / Non-Goals

**Goals:**
- Commands with `;`, `&&`, `||` are split correctly: first segment used for plugin matching, rest goes to bash
- `$?` and other shell variables are expanded correctly in commands with metacharacters
- Plugins still match against the first command segment
- Quote-aware: metacharacters inside `'...'` or `"..."` are ignored

**Non-Goals:**
- No changes to `shlex` crate
- No changes to individual plugins (rtk, jq, sed, etc.)
- No full shell parser — only handle `;`, `&&`, `||` (the most common cases)

## Decisions

### 1. Quote-aware metacharacter detection

Reuse the state machine from `split_pipeline()`:

```rust
fn has_shell_metacharacters(input: &str) -> Option<&str> {
    // Returns Some(metachar) if found outside quotes, None otherwise
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escaped = false;
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if escaped { escaped = false; i += 1; continue; }
        if c == '\\' && !in_single_quote { escaped = true; i += 1; continue; }
        if c == '\'' && !in_double_quote { in_single_quote = !in_single_quote; i += 1; continue; }
        if c == '"' && !in_single_quote { in_double_quote = !in_double_quote; i += 1; continue; }
        if !in_single_quote && !in_double_quote {
            if c == ';' { return Some(";"); }
            if c == '&' && i + 1 < chars.len() && chars[i + 1] == '&' { return Some("&&"); }
            if c == '|' && i + 1 < chars.len() && chars[i + 1] == '|' { return Some("||"); }
        }
        i += 1;
    }
    None
}
```

### 2. Split on first metacharacter

When a metacharacter is found, split the command into first segment and rest:

```rust
fn split_first_command(input: &str) -> (&str, &str) {
    // Returns (first_command, rest)
    // Uses same state machine to find the split point
}
```

### 3. Modified dispatch_full flow

```
dispatch_full(full_cmd)
  │
  ├── Check for shell metacharacters outside quotes
  │
  ├── If found:
  │     ├── Split: (first_segment, rest)
  │     ├── Match plugins against first_segment
  │     ├── If plugin matches:
  │     │     ├── Run plugin with first_segment
  │     │     ├── Run rest via execute_passthrough
  │     │     └── Concatenate outputs
  │     └── If no plugin matches:
  │           └── Run full_cmd via execute_passthrough
  │
  └── If not found:
        └── Normal dispatch (existing behavior)
```

### 4. Edge cases

| Input | First segment | Rest | Behavior |
|-------|---------------|------|----------|
| `wc -c; echo $?` | `wc -c` | ` echo $?` | rtk matches `wc`, rest goes to bash |
| `sed -n '1,5p' f; echo done` | `sed -n '1,5p' f` | ` echo done` | sed matches, rest goes to bash |
| `echo "hello; world"` | (no split) | — | `;` inside quotes, ignored |
| `cmd1 && cmd2` | `cmd1` | ` cmd2` | First segment matched against plugins |
| `cmd1 \|\| cmd2` | `cmd1` | ` cmd2` | First segment matched against plugins |
| `echo ';'` | (no split) | — | `;` inside quotes, ignored |

## Risks / Trade-offs

- **First-segment-only matching**: Only the first command before the metacharacter gets plugin processing. Subsequent commands always go to bash. This is correct for the common case (e.g., `cmd; echo done`) but means `cmd1; cmd2` won't match plugins for `cmd2`.
- **No `|` handling**: Pipes (`|`) are already handled by `try_pipeline()`. The metacharacter detection explicitly excludes `|` to avoid conflict.
- **No `&` handling**: Backgrounding (`&`) is not handled. It's rare in agent contexts and would add complexity.
