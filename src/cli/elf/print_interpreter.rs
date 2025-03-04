use std::path::PathBuf;

use clap::Parser;

/// Print the interpreter of the ELF file.
#[derive(Parser, Debug)]
pub struct Args {
    /// Path to the file to change
    pub path_to_binary: PathBuf,
}

pub fn execute(args: Args) -> Result<(), crate::elf::ElfError> {
    let bytes_of_file = std::fs::read(&args.path_to_binary).unwrap();

    let mut elf = crate::elf::ElfContainer::parse(&bytes_of_file)?;

    elf.print_interpreter();

    Ok(())
}
