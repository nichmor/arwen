use std::path::PathBuf;

use clap::Parser;

/// Adds `DT_DEBUG` tag to the elf file to enable debugging
#[derive(Parser, Debug)]
pub struct Args {
    /// Path to the file to change
    pub path_to_binary: PathBuf,
}

pub fn execute(args: Args) -> Result<(), arwen_elf::ElfError> {
    super::patch(&args.path_to_binary, |elf| {
        elf.add_debug_tag()?;
        Ok(())
    })
}
