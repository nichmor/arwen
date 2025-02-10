use std::path::PathBuf;

use clap::Parser;

use crate::macho::{MachoContainer, MachoError};

/// Change already existing run path
#[derive(Parser, Debug)]
pub struct Args {
    /// Old rpath to remove
    pub old_rpath: String,

    /// New rpath to add
    pub new_rpath: String,

    /// Path to the file to change
    pub path_to_binary: PathBuf,
}

pub fn execute(args: Args) -> Result<(), MachoError> {
    let bytes_of_file = std::fs::read(&args.path_to_binary).unwrap();

    let mut macho = MachoContainer::parse(&bytes_of_file)?;

    macho.change_rpath(&args.old_rpath, &args.new_rpath);

    std::fs::write(args.path_to_binary, macho.data).unwrap();

    Ok(())
}
