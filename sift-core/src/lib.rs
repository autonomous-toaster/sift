//! Core types and runtime for sift.
//!
//! This crate defines the session store, command classification,
//! and Lua plugin runtime — the foundation that the sift binary builds on.

#![deny(clippy::all, clippy::pedantic, clippy::nursery)]
#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![allow(dead_code, clippy::missing_errors_doc)]

pub mod classifier;
pub mod lua;
pub mod session;
