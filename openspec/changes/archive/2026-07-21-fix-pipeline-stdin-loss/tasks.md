## 1. Skip wildcard plugins in pipeline optimization

- [x] 1.1 In `try_pipeline()`, after finding the matched plugin via `find_plugin()`, check if its first pattern is `"*"` and return `None` if so
- [x] 1.2 Add unit test: pipeline with wildcard-matched command (`sort`) runs full pipeline in bash, produces correct output
- [x] 1.3 Add unit test: pipeline with specific plugin match (`cat`) still triggers optimization

## 2. Forward stdin in execute_passthrough

- [x] 2.1 Change `execute_passthrough()` signature to accept optional stdin: `fn execute_passthrough(cmd, args, stdin)`
- [x] 2.2 In `dispatch()`, when status is `"passthrough"`, read the `StdinReader` content (if present) and pass it to `execute_passthrough()`
- [x] 2.3 Add unit test: passthrough with piped input forwards stdin correctly
- [x] 2.4 Add unit test: passthrough without stdin (non-pipeline mode) behaves as before
