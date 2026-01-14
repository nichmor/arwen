use crate::commands::CommandBuilderError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MachoError {
    #[error("error while parsing Mach-O file")]
    Parsing(#[from] goblin::error::Error),

    #[error("unix archive is not supported as fat file format")]
    UnixArchive,

    #[error("could not determine endianness of the file")]
    UnknownEndian,

    #[error("expected more fat arches than declared in header")]
    FatArch,

    #[error("error when creating a new command: {0}")]
    CommandBuilderError(#[from] CommandBuilderError),

    #[error("error when writing struct into bytes: {0}")]
    WritingError(#[from] scroll::Error),

    #[error("requested rpath is missing: {0}")]
    RpathMissing(String),

    #[error("requested dylib name is missing: {0}")]
    DylibNameMissing(String),

    #[error("LC_ID_DYLIB is missing or file is not a shared library")]
    DylibIdMissing,

    #[error("codesign section is missing")]
    CodesignMissing,
}
