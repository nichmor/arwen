use std::path::PathBuf;

use clap::Parser;

/// Set the interpreter of the elf file
#[derive(Parser, Debug)]
pub struct Args {
    /// Interpreter to set
    pub interpreter: String,

    /// Path to the file to change
    pub path_to_binary: PathBuf,
}

pub fn execute(args: Args) -> Result<(), arwen_elf::ElfError> {
    super::patch(&args.path_to_binary, |elf| {
        elf.set_interpreter(&args.interpreter)?;
        Ok(())
    })
}
