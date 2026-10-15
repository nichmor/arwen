use std::path::PathBuf;

use clap::Parser;

/// Sets the DT_SONAME field of a shared library (fails for files that are not ET_DYN)
#[derive(Parser, Debug)]
pub struct Args {
    /// SONAME to set
    pub soname: String,

    /// Path to the file to change
    pub path_to_binary: PathBuf,
}

pub fn execute(args: Args) -> Result<(), arwen_elf::ElfError> {
    super::patch(&args.path_to_binary, |elf| {
        elf.set_soname(&args.soname)?;
        Ok(())
    })
}
