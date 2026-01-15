//! ELF binary patching library.
//!
//! This crate provides tools for reading and modifying ELF binaries,
//! including operations on runpaths, interpreters, sonames, and more.

pub mod container;
pub mod rewriter;

pub use container::*;
pub use rewriter::*;
