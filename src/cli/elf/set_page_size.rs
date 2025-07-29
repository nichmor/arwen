use std::path::PathBuf;

use clap::Parser;

/// Set the page size for ELF file segment alignment
#[derive(Parser, Debug)]
pub struct Args {
    /// Page size in bytes (must be power of 2 and >= 1024)
    pub page_size: u32,

    /// Path to the file to change
    pub path_to_binary: PathBuf,
}

pub fn execute(args: Args) -> Result<(), crate::elf::ElfError> {
    let bytes_of_file = std::fs::read(&args.path_to_binary).unwrap();

    let mut elf = crate::elf::ElfContainer::parse(&bytes_of_file)?;

    elf.set_page_size(args.page_size)?;

    let output_file =
        std::fs::File::create(format!("{}", args.path_to_binary.to_string_lossy())).unwrap();

    elf.write(&output_file)?;

    Ok(())
}
