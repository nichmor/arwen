use std::path::PathBuf;

use clap::Parser;

use crate::macho::MachoContainer;


/// Remove a run path
#[derive(Parser, Debug)]
pub struct Args {
    /// Path to the file to remove
    #[arg(short)]
    pub path: PathBuf,

    /// Rpath to remove
    #[arg(short)]
    pub rpath: String,
}

pub fn execute(args: Args) {
    let bytes_of_file = std::fs::read(&args.path).unwrap();

    let mut macho = MachoContainer::parse(&bytes_of_file);

    macho.remove_rpath(&args.rpath);

    let new_path = args.path.with_file_name(format!(
        "{}_no_rpath",
        args.path.file_name().unwrap().to_str().unwrap()
    ));

    std::fs::write(new_path, macho.data).unwrap();
}
