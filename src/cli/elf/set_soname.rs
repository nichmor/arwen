use std::path::PathBuf;

use clap::Parser;

/// Sets the DT_SOABI field in the ELF file
#[derive(Parser, Debug)]
pub struct Args {
    /// SONAME to set
    pub soname: String,

    /// Path to the file to change
    pub path_to_binary: PathBuf,
}

pub fn execute(args: Args) -> Result<(), crate::macho::MachoError> {
    let bytes_of_file = std::fs::read(&args.path_to_binary).unwrap();

    let mut elf = crate::elf::ElfContainer::parse(&bytes_of_file)?;

    elf.set_soname(&args.soname)?;

    let output_file =
        std::fs::File::create(format!("{}_patched", args.path_to_binary.to_string_lossy()))
            .unwrap();

    elf.write(&output_file)?;

    Ok(())
}
