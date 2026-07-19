## Context

`sift gain` was implemented as a Lua plugin. The user wants a CLI flag instead. The `record_conversation` function uses `Handle::current().block_on()` which panics when called from within a tokio runtime (the `#[tokio::main]` context). Clippy denies `unwrap_used`, `expect_used`, `panic` were added to Cargo.toml but violations remain.

## Goals / Non-Goals

**Goals:**
- `sift --gain` reads session store and prints gain report, then exits
- Remove `plugins/gain.lua`
- Fix all remaining clippy violations
- Add Justfile recipe to verify lint rules

**Non-Goals:**
- Changing the gain report format or data model
- Adding new features to the gain report

## Decisions

1. **CLI flag, not subcommand** — `sift --gain` is simpler than `sift gain` as a subcommand. The existing CLI uses clap with `-c` and `--shell` flags. Adding `--gain` follows the same pattern.

2. **`sift.gain.report()` stays** — The Rust-registered Lua function remains for programmatic access from other plugins. The CLI flag is the primary user-facing entry point.

3. **Fix clippy violations by extracting helpers** — No `#[allow(...)]` attributes. Extract functions where needed.

## Risks / Trade-offs

1. **`sift --gain` without AI_SESSION** — Shows a message guiding the user to set AI_SESSION. No crash.