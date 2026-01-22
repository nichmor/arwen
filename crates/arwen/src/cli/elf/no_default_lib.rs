use std::path::PathBuf;

use clap::Parser;

/// Disable the default library search paths in the elf file
#[derive(Parser, Debug)]
pub struct Args {
    /// Path to the file to change
    pub path_to_binary: PathBuf,
}

pub fn execute(args: Args) -> Result<(), arwen_elf::ElfError> {
    let bytes_of_file = std::fs::read(&args.path_to_binary).unwrap();

    let mut elf = arwen_elf::ElfContainer::parse(&bytes_of_file)?;

    elf.no_default_lib()?;

    let output_file =
        std::fs::File::create(format!("{}", args.path_to_binary.to_string_lossy())).unwrap();

    elf.write(&output_file)?;

    Ok(())
}
