use std::path::PathBuf;

use clap::Parser;

/// Add dependencies on a dynamic library  DT_NEEDED
#[derive(Parser, Debug)]
pub struct Args {
    /// Path to the file to change
    pub path_to_binary: PathBuf,

    /// New DT_NEEDED to add
    pub dt_needed: Vec<String>,
}

pub fn execute(args: Args) -> Result<(), arwen_elf::ElfError> {
    super::patch(&args.path_to_binary, |elf| {
        elf.add_needed(args.dt_needed)?;
        Ok(())
    })
}
