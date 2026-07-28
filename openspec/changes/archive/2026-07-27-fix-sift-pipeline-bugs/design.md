## Context

Sift's command dispatch flow for a piped command like `curl ... | jq ...`:

```
dispatch_full(cmd)
  ├─ try_pipeline()          ← pipeline optimization
  │   ├─ split_pipeline()    ← naive | splitter (BUG: quote-unaware)
  │   ├─ find_plugin(last)   ← match last segment to plugin
  │   ├─ if no match → return None (BUG: falls to broken path)
  │   └─ if match → run preceding in bash, pipe to plugin
  └─ normal dispatch         ← splits by whitespace, routes to plugin
      └─ bash.lua shell-quotes everything → | becomes literal arg
```

The agent mode exit path:
```
agent_mode() → dispatch_full() → ... → print!() + flush() → return
                                                          ↓
                                              std::process::exit()
                                              (BUG: may not flush stdout)
```

## Goals / Non-Goals

**Goals:**
1. All piped commands produce correct output visible to the caller
2. `curl ... | jq ...` works reliably in both agent and REPL mode
3. `curl ... | grep ...` (unmatched last segment) falls back to bash correctly
4. URLs or strings containing `|` inside quotes are not split

**Non-Goals:**
- Adding new plugins for every possible piped command
- Rewriting the entire dispatch system
- Changing the plugin API

## Decisions

### Decision 1: Flush stdout before process exit
- **Where**: `sift/src/main.rs` — add `std::io::stdout().flush()` before `std::process::exit()`
- **Why**: `std::process::exit()` terminates immediately without flushing buffers. The `print!` output from `dispatch()` may still be buffered.
- **Also**: Flush in `agent_mode()` after `dispatch_full()` returns, as belt-and-suspenders.

### Decision 2: Make `split_pipeline` quote-aware
- **Where**: `sift-core/src/lua/api.rs` — replace `split_pipeline()`
- **How**: Track whether we're inside single quotes, double quotes, or escaped characters. Only split on `|` when outside all quoting contexts.
- **Why**: A URL like `https://api.example.com?filter=a|b` inside quotes must not be split.

### Decision 3: Pipeline fallback for unmatched segments
- **Where**: `sift-core/src/lua/api.rs` — in `try_pipeline()`, when `find_plugin()` returns `None`
- **How**: Instead of returning `None` (which falls to broken normal dispatch), run the entire pipeline through `exec_command()` in bash.
- **Why**: The normal dispatch path cannot handle pipes — it treats `|` as a literal argument. Running the full pipeline in bash is the correct fallback.

## Risks / Trade-offs

- **Risk**: The quote-aware `split_pipeline` might miss edge cases (backslash escapes, $() subshells). Mitigation: use a state machine with clear transitions, add tests for all quote styles.
- **Risk**: Running the full pipeline in bash for unmatched segments loses sift's optimization (caching, compression). Trade-off: correctness over optimization for uncommon commands.
- **Risk**: Flushing stdout twice (in `dispatch()` and before `exit()`) is harmless but redundant. Acceptable for belt-and-suspenders safety.
