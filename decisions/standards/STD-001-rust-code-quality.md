# STD-001 · Rust Code Quality

## Lint configuration

All crates must declare at the crate root (`lib.rs` / `main.rs`):

```rust
#![deny(clippy::all, clippy::pedantic, clippy::nursery)]
#![deny(missing_docs)]
#![forbid(unsafe_code)]
```

Workspace-level defaults in `Cargo.toml`:

```toml
[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.lints.clippy]
all = "deny"
pedantic = "deny"
nursery = "deny"
```

## Forbidden patterns (production code)

| Pattern | Policy |
|---|---|
| `unwrap()` | Forbidden |
| `expect()` | Forbidden |
| `panic!()` | Forbidden |
| `unsafe` | Forbidden (`forbid`) |
| `dbg!()` | Forbidden |

All of the above are freely allowed in `#[cfg(test)]` and `tests/` scopes.

## Error handling

All fallible functions must return `Result<T, E>` where `E` implements `std::error::Error`. Use `anyhow` for binary code, `thiserror` for library code. Never use `unwrap()` or `expect()` in production paths — propagate errors instead.

## Dependency auditing

`cargo-deny` is required. A `deny.toml` must be present at the workspace root checking security advisories and duplicate crates. `cargo deny check` must pass in CI before any build step.
