use std::path::PathBuf;

use clap::Parser;

/// Change the OS ABI of the elf file
#[derive(Parser, Debug)]
pub struct Args {
    /// OS ABI to set (e.g. `linux`, `freebsd`, `system v`)
    pub os_abi: String,

    /// Path to the file to change
    pub path_to_binary: PathBuf,
}

pub fn execute(args: Args) -> Result<(), arwen_elf::ElfError> {
    super::patch(&args.path_to_binary, |elf| {
        elf.set_os_abi(&args.os_abi)?;
        Ok(())
    })
}
