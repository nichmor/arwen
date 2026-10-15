use std::path::PathBuf;

use clap::Parser;

/// Clears the executable flag of the GNU_STACK program header.
#[derive(Parser, Debug)]
pub struct Args {
    /// Path to the file to change
    pub path_to_binary: PathBuf,
}

pub fn execute(args: Args) -> Result<(), arwen_elf::ElfError> {
    super::patch(&args.path_to_binary, |elf| {
        elf.clear_exec_stack()?;
        Ok(())
    })
}
