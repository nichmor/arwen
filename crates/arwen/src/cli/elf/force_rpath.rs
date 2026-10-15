use std::path::PathBuf;

use clap::Parser;

/// Force usage of `DT_RPATH` instead of `DT_RUNPATH`.
#[derive(Parser, Debug)]
pub struct Args {
    /// Path to the file to change
    pub path_to_binary: PathBuf,
}

pub fn execute(args: Args) -> Result<(), arwen_elf::ElfError> {
    super::patch(&args.path_to_binary, |elf| {
        elf.force_rpath()?;
        Ok(())
    })
}
