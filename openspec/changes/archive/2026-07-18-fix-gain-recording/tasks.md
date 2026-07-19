## 1. Fix exit race condition

- [x] 1.1 Change `agent_mode()` to return `Result<i32>` instead of calling `std::process::exit()`
- [x] 1.2 In `main()`, capture exit code from `agent_mode()` and call `std::process::exit()` after the async context returns