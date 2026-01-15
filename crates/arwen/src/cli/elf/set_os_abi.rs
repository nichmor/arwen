use std::path::PathBuf;

use clap::Parser;

/// Change the OS ABI of the elf file
#[derive(Parser, Debug)]
pub struct Args {
    /// Interpreter to set
    pub os_abi: String,

    /// Path to the file to change
    pub path_to_binary: PathBuf,
}

pub fn execute(args: Args) -> Result<(), arwen_elf::ElfError> {
    let bytes_of_file = std::fs::read(&args.path_to_binary).unwrap();

    let mut elf = arwen_elf::ElfContainer::parse(&bytes_of_file)?;

    elf.set_os_abi(&args.os_abi)?;

    let output_file =
        std::fs::File::create(format!("{}", args.path_to_binary.to_string_lossy())).unwrap();

    elf.write(&output_file)?;

    Ok(())
}
