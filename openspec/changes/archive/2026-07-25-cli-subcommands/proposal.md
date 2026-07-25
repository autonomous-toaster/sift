# CLI Subcommands

## Summary

Restructure sift's CLI from flat flags to subcommands: `sift gain` replaces `sift --gain`, REPL becomes the default mode, and gain tracking data can be reset.

## Motivation

The current flat-flag CLI (`--gain`, `--shell`, `-c`) is awkward as features grow. `--gain` has accumulated 7+ flags that all hang off a single boolean. Making REPL the default removes a dead-end UX (`sift` with no args does nothing). Adding `--reset` gives users control over gain data.

## Scope

- Convert `--gain` flag to `sift gain` subcommand with all gain flags under it
- Make REPL the default mode (no args → REPL)
- Remove `--gain` flag entirely (no backward compat)
- Add `--reset` flag to `sift gain` for clearing gain data
- Store full reconstructed command (name + args) instead of just first token
- Keep `-c` for agent mode, `--shell` for explicit REPL

## Out of Scope

- Changes to the gain report format or aggregation logic
- Changes to plugin dispatch or Lua API
- Changes to session store schema
