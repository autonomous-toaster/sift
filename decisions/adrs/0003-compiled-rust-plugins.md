---
status: proposed
date: 2026-07-11
---

# Compiled Rust Plugins (not Lua, not Dynamic Loading)

## Context and Problem Statement

How should plugins be defined and loaded? Plugins wrap individual commands (cat, git, cargo, curl) and provide optimized execution, caching, and fingerprinting.

## Considered Options

* Compiled Rust plugins — plugins are Rust structs implementing a `Plugin` trait, registered at compile time.
* Lua scripting — plugins are Lua scripts loaded at runtime via mlua or rlua.
* Dynamic loading — plugins are shared libraries (.so/.dylib) loaded via libloading.
* Subprocess — plugins are separate binaries called by baish.

## Decision Outcome

Chosen option: Compiled Rust plugins, because Lua adds a heavy dependency (luajit, FFI, sandboxing) with no clear benefit for a shell that already controls execution. Dynamic loading adds ABI stability concerns. Subprocess adds overhead for every command. Compiled Rust plugins are type-safe, zero-overhead, and co-located with the codebase.

### Consequences

* Good, because plugins are type-safe — the compiler catches mismatches between plugin expectations and actual command structure.
* Good, because there is zero runtime overhead — plugin dispatch is a simple trait method call.
* Good, because plugins have full access to the Rust ecosystem (serde, regex, sha2, etc.).
* Bad, because adding a new plugin requires recompiling baish. Acceptable for a shell binary that ships as a single artifact.
* Bad, because third-party plugins require forking baish or contributing upstream. Acceptable for v1 — plugin ecosystem can be addressed later.
