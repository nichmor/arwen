use std::path::PathBuf;

use clap::Parser;

/// Add a run path to the elf file
#[derive(Parser, Debug)]
pub struct Args {
    /// Interpreter to set
    pub interpreter: String,

    /// Path to the file to change
    pub path_to_binary: PathBuf,
}

pub fn execute(args: Args) -> Result<(), crate::macho::MachoError> {
    let bytes_of_file = std::fs::read(&args.path_to_binary).unwrap();

    let mut elf = crate::elf::ElfContainer::parse(&bytes_of_file)?;

    elf.set_interpreter(&args.interpreter)?;

    let output_file =
        std::fs::File::create(format!("{}_patched", args.path_to_binary.to_string_lossy()))
            .unwrap();

    elf.write(&output_file)?;

    // std::fs::write(args.path_to_binary, macho.data).unwrap();

    Ok(())
}
