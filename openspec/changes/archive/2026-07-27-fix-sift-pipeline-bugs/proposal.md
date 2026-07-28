## Why

Sift's pipeline optimization (`try_pipeline`) has three bugs that cause incorrect or missing output for piped commands:

1. **Stdout not flushed in agent mode** — `sift -c "curl ... | jq ..."` produces no visible output because `print!` output is not flushed before `std::process::exit()`. The output exists but is lost in the buffer.

2. **`split_pipeline` is quote-unaware** — The naive `|` splitter doesn't handle quoted strings. A URL containing `|` (e.g., `curl 'https://api.example.com?filter=a|b'`) would be split incorrectly, breaking the pipeline.

3. **No fallback for unmatched pipeline segments** — When the last pipeline segment has no matching plugin (e.g., `curl ... | grep foo`), `try_pipeline` returns `None` and the normal dispatch treats `|` as a literal argument — shell-quoted and passed to the first command instead of being a shell pipe.

These bugs affect all users running piped commands through sift, especially common patterns like `curl | jq`, `curl | grep`, and `docker ps | awk`.

## What Changes

1. **Fix stdout flush in agent mode** — Ensure `std::io::stdout().flush()` is called before `std::process::exit()` in the agent mode path.

2. **Make `split_pipeline` quote-aware** — Replace the naive character-by-character split with a parser that respects single quotes, double quotes, and escaped characters.

3. **Add pipeline fallback for unmatched segments** — When the last pipeline segment has no matching plugin, run the entire pipeline through bash instead of falling through to the broken normal dispatch path.

## Capabilities

### New Capabilities
- `pipeline-fix`: Correct handling of shell pipelines through sift, including proper quoting, fallback to bash for unmatched commands, and reliable output delivery.

### Modified Capabilities
*(No existing capability specs are changing — this is a bugfix within the existing sift dispatch system.)*

## Impact

- **sift-core/src/lua/api.rs**: `try_pipeline()`, `split_pipeline()`, `dispatch_full()`
- **sift/src/main.rs**: `agent_mode()`, `main()` exit path
- **sift-core/src/lua/tests.rs**: Pipeline-related tests
- **sift-core/src/lua/tests_plugins.rs**: jq plugin pipeline tests
