use std::path::PathBuf;

use clap::Parser;

/// Disable the default library search paths in the elf file
#[derive(Parser, Debug)]
pub struct Args {
    /// Path to the file to change
    pub path_to_binary: PathBuf,
}

pub fn execute(args: Args) -> Result<(), crate::macho::MachoError> {
    let bytes_of_file = std::fs::read(&args.path_to_binary).unwrap();

    let mut elf = crate::elf::ElfContainer::parse(&bytes_of_file)?;

    elf.no_default_lib()?;

    let output_file =
        std::fs::File::create(format!("{}_patched", args.path_to_binary.to_string_lossy()))
            .unwrap();

    elf.write(&output_file)?;

    // std::fs::write(args.path_to_binary, macho.data).unwrap();

    Ok(())
}
