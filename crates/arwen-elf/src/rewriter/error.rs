use object::build;
use std::io;
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Debug, Error)]
/// An error that occurred while reading, modifying, or writing an ELF file.
pub enum ElfError {
    #[error("I/O error occurred while writing the file")]
    /// An I/O error occurred while writing the file.
    Io(#[from] io::Error),
    #[error("A parse error occurred while reading the file")]
    /// A parse error occurred while reading the file.
    Parse(#[from] build::Error),
    #[error("A validation error occurred while writing the file")]
    /// An error occurred while writing the file.
    Write(build::Error),
    #[error("cannot modify the file: {0}")]
    /// The modification cannot be applied to this file.
    Modify(String),
    #[error("cannot find section '{0}'")]
    /// The file has no section that the operation needs.
    MissingSection(&'static str),
    #[error("Non-UTF-8 DT_NEEDED entry")]
    /// The `DT_NEEDED` entry is not UTF-8 encoded
    InvalidDtNeededEncoding(#[source] FromUtf8Error),
}

/// The  `Result` type for this library.
pub type Result<T> = std::result::Result<T, ElfError>;
