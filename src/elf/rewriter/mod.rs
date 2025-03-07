//! This module provides a way to rewrite ELF files.
//!
//! The main struct is `Rewriter` which provides methods to read, modify, and write ELF files.
//!
//! `BuilderExt` is a trait that extends the `object::Builder` struct with additional methods.
//!
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

mod error;
pub use error::{ElfError, Result};

mod writer;
pub use writer::Writer;

mod elf;

mod ext;
pub use ext::BuilderExt;
