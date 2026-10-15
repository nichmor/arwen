use std::path::PathBuf;

use clap::Parser;

/// Remove dynamic library dependencies from DT_NEEDED
#[derive(Parser, Debug)]
pub struct Args {
    /// Path to the file to change
    pub path_to_binary: PathBuf,

    /// New DT_NEEDED to remove
    pub dt_needed: Vec<String>,
}

pub fn execute(args: Args) -> Result<(), arwen_elf::ElfError> {
    super::patch(&args.path_to_binary, |elf| {
        elf.remove_needed(args.dt_needed)?;
        Ok(())
    })
}
