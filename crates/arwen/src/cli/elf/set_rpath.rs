use std::path::PathBuf;

use clap::Parser;

/// Set the rpath of the elf file
#[derive(Parser, Debug)]
pub struct Args {
    /// New rpath to add
    pub new_rpath: String,

    /// Path to the file to change
    pub path_to_binary: PathBuf,
}

pub fn execute(args: Args) -> Result<(), arwen_elf::ElfError> {
    super::patch(&args.path_to_binary, |elf| {
        elf.set_runpath(&args.new_rpath)?;
        Ok(())
    })
}
