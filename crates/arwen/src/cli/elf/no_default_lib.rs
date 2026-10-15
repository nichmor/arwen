use std::path::PathBuf;

use clap::Parser;

/// Disable the default library search paths in the elf file
#[derive(Parser, Debug)]
pub struct Args {
    /// Path to the file to change
    pub path_to_binary: PathBuf,
}

pub fn execute(args: Args) -> Result<(), arwen_elf::ElfError> {
    super::patch(&args.path_to_binary, |elf| {
        elf.no_default_lib()?;
        Ok(())
    })
}
