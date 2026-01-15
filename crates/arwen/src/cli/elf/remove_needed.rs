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
    let bytes_of_file = std::fs::read(&args.path_to_binary).unwrap();

    let mut elf = arwen_elf::ElfContainer::parse(&bytes_of_file)?;

    elf.remove_needed(args.dt_needed)?;

    let output_file =
        std::fs::File::create(format!("{}", args.path_to_binary.to_string_lossy())).unwrap();

    elf.write(&output_file)?;

    Ok(())
}
