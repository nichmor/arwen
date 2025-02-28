use std::path::PathBuf;

use clap::Parser;

/// Add dependencies on a dynamic library  DT_NEEDED
#[derive(Parser, Debug)]
pub struct Args {
    /// Path to the file to change
    pub path_to_binary: PathBuf,

    /// New DT_NEEDED to add
    pub dt_needed: Vec<String>,
}

pub fn execute(args: Args) -> Result<(), crate::macho::MachoError> {
    let bytes_of_file = std::fs::read(&args.path_to_binary).unwrap();

    let mut elf = crate::elf::ElfContainer::parse(&bytes_of_file)?;

    elf.add_needed(args.dt_needed)?;

    let output_file =
        std::fs::File::create(format!("{}_patched", args.path_to_binary.to_string_lossy()))
            .unwrap();

    elf.write(&output_file)?;

    // std::fs::write(args.path_to_binary, macho.data).unwrap();

    Ok(())
}
