# STD-002 · Project Layout and Crate Design

## Workspace structure

The project is a Cargo workspace. All crates live under `src/`:

```
baish/
├── Cargo.toml              # workspace root
├── src/
│   ├── main.rs             # Entry point, REPL loop
│   ├── parser.rs           # Wrapper around brush-parser
│   ├── dispatcher.rs       # Walk AST, dispatch to plugins or exec
│   ├── session.rs          # Session store (SQLite via sqlx)
│   ├── plugin.rs           # Plugin trait + registry
│   ├── builtins.rs         # cd, export, unset, source, exit
│   └── plugins/
│       ├── mod.rs
│       └── cat.rs           # CatPlugin
├── decisions/
│   ├── adrs/               # Architecture Decision Records
│   └── standards/          # Project standards
├── openspec/               # OpenSpec artifacts
│   ├── config.yaml
│   ├── specs/              # Capability specs
│   └── changes/            # Active changes
└── tests/                  # Integration tests
```

## Design principles

**Single binary, no library split.** baish is a standalone shell binary. Internal modules are organized by concern, not published as separate crates. If a module grows large enough to warrant independent testing, extract it into a workspace crate under `src/`.

**Module boundaries follow responsibility.**
- `parser.rs` — parse shell input into AST. Thin wrapper over brush-parser.
- `dispatcher.rs` — walk AST, decide plugin vs exec, handle pipes. Core orchestration.
- `session.rs` — SQLite-backed session store. All DB access goes through this module.
- `plugin.rs` — Plugin trait, registry, and dispatch logic.
- `builtins.rs` — Shell builtins (cd, export, unset, source, exit).
- `plugins/` — One file per plugin. Each plugin implements the Plugin trait.

**Dependency direction is strict:**
```
main → dispatcher → parser
main → dispatcher → plugin
main → dispatcher → builtins
dispatcher → session
plugins → session
```

`session` must not depend on `plugins`, `dispatcher`, or `builtins`. `parser` must not depend on anything except brush-parser.

**Traits over concrete types at boundaries.** `Plugin` is a trait. The registry holds `Box<dyn Plugin>`. Plugins are registered at startup in `main.rs`.

**No business logic in `main.rs`.** `main.rs` sets up the session, registers plugins, and runs the REPL loop. All decision logic lives in `dispatcher.rs`.
