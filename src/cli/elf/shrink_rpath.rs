use std::path::PathBuf;

use clap::Parser;

/// Remove from the DT_RUNPATH or DT_RPATH all directories that do not contain a
/// library referenced by DT_NEEDED fields of the executable or library.
/// For instance, if an executable references one library libfoo.so,
/// has an RPATH "/lib:/usr/lib:/foo/lib", and libfoo.so can only be found in /foo/lib, then the new
/// RPATH will be "/foo/lib".
#[derive(Parser, Debug)]
pub struct Args {
    /// Path to the file to change
    pub path_to_binary: PathBuf,

    /// option to allow rpath prefixes
    #[clap(long, short)]
    pub allowed_rpath_prefixes: Vec<String>,
}

pub fn execute(args: Args) -> Result<(), crate::macho::MachoError> {
    let bytes_of_file = std::fs::read(&args.path_to_binary).unwrap();

    let mut elf = crate::elf::ElfContainer::parse(&bytes_of_file)?;

    elf.shrink_rpath(args.allowed_rpath_prefixes)?;

    let output_file =
        std::fs::File::create(format!("{}", args.path_to_binary.to_string_lossy())).unwrap();

    elf.write(&output_file)?;

    // std::fs::write(args.path_to_binary, macho.data).unwrap();

    Ok(())
}
