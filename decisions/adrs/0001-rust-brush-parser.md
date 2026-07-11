---
status: proposed
date: 2026-07-11
---

# Rust + brush-parser as Language and Parser

## Context and Problem Statement

What language and shell parser should be used to implement baish, an AI-optimized shell that replaces bash for LLM coding agents?

## Considered Options

* Rust + brush-parser (MIT)
* Rust + yash-syntax (GPL-3.0)
* Go + custom parser
* C + existing shell library

## Decision Outcome

Chosen option: Rust + brush-parser, because Rust provides memory safety without a garbage collector, zero-cost abstractions, and a type system that enforces correctness at compile time — critical for a shell that replaces bash in production agent workflows. brush-parser (MIT license) provides a full POSIX/bash AST including pipes, redirects, heredocs, subshells, and compound commands, with no license compatibility concerns.

### Consequences

* Good, because brush-parser handles the full POSIX shell syntax — pipes (`|`), redirects (`>`, `>>`, `<`, `2>&1`), operators (`&&`, `||`, `;`), subshells (`$(...)`), heredocs (`<<EOF`), globs (`*.rs`), and env vars (`$VAR`).
* Good, because MIT license avoids GPL contamination concerns (unlike yash-syntax's GPL-3.0).
* Good, because brush-parser is part of the `brush` project (a full bash-compatible shell in Rust), ensuring ongoing maintenance and compatibility.
* Good, because Rust's `std::process::Command` and `os_pipe` crate provide ergonomic pipe and subprocess management.
* Bad, because brush-parser is pre-1.0 (v0.4.0) — API may change.
* Bad, because compile times are longer than Go or C alternatives.
