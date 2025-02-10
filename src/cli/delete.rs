use std::path::PathBuf;

use clap::Parser;

use crate::macho::{MachoContainer, MachoError};

/// Remove a run path
#[derive(Parser, Debug)]
pub struct Args {
    /// Rpath to remove
    pub rpath_to_remove: String,

    /// Path to the file to change
    pub path_to_binary: PathBuf,
}

pub fn execute(args: Args) -> Result<(), MachoError> {
    let bytes_of_file = std::fs::read(&args.path_to_binary).unwrap();

    let mut macho = MachoContainer::parse(&bytes_of_file)?;

    macho.remove_rpath(&args.rpath_to_remove)?;

    std::fs::write(args.path_to_binary, macho.data).unwrap();

    Ok(())
}
