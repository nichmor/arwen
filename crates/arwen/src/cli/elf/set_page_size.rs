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

pub fn execute(args: Args) -> Result<(), arwen_elf::ElfError> {
    super::patch(&args.path_to_binary, |elf| {
        elf.set_page_size(args.page_size)?;
        Ok(())
    })
}
