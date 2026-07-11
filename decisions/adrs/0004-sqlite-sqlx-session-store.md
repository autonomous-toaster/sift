---
status: proposed
date: 2026-07-11
---

# SQLite via sqlx for Session Store

## Context and Problem Statement

baish needs a session store to track file hashes, conversation cache, and fingerprints across commands within an AI agent session. The store must be lightweight, embedded (no external server), and accessible from Rust.

## Considered Options

* SQLite via sqlx — embedded SQL database with compile-time checked queries.
* SQLite via rusqlite — embedded SQL database, synchronous API.
* In-memory HashMap — no persistence, lost on restart.
* JSON file — simple file-based storage.

## Decision Outcome

Chosen option: SQLite via sqlx, because it provides a structured, queryable store with compile-time query checking, async support, and no external server. The database file lives at `.baish/session_<AI_SESSION_ID>.db` and is created on demand. Only one connection is used (no pool needed for a single-user shell).

### Consequences

* Good, because sqlx provides compile-time checked SQL queries — malformed queries are caught at build time.
* Good, because SQLite is embedded — no external process, no configuration.
* Good, because the schema is trivially inspectable with any SQLite tool.
* Good, because sqlx supports migrations for schema evolution.
* Bad, because sqlx's SQLite driver is async-only — requires `block_on` or an async runtime in the synchronous shell context. Acceptable — Tokio's `block_on` is well-suited for this.
* Bad, because sqlx adds ~20 dependencies. Acceptable for a shell binary.
