use std::path::PathBuf;

use clap::Parser;

/// Clear a version for a specific symbol
#[derive(Parser, Debug)]
pub struct Args {
    /// Symbol to clear the version for
    pub symbol: String,

    /// Path to the file to change
    pub path_to_binary: PathBuf,
}

pub fn execute(args: Args) -> Result<(), arwen_elf::ElfError> {
    super::patch(&args.path_to_binary, |elf| {
        elf.clear_version_symbol(&args.symbol)?;
        Ok(())
    })
}
