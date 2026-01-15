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
    #[error("A validation error occurred while modifying the file")]
    /// An error occurred while modifying the file.
    Modify(String),
    #[error("Non-UTF-8 DT_NEEDED entry")]
    /// The `DT_NEEDED` entry is not UTF-8 encoded
    InvalidDtNeededEncoding(#[source] FromUtf8Error),
}

/// The  `Result` type for this library.
pub type Result<T> = std::result::Result<T, ElfError>;
