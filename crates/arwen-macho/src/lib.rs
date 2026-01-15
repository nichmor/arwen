//! Mach-O binary patching library.
//!
//! This crate provides tools for reading and modifying Mach-O binaries,
//! including operations on rpaths, install names, and dylib IDs.

pub mod commands;
pub mod container;
pub mod error;
pub mod patcher;
mod utils;

pub use container::*;
pub use error::MachoError;
