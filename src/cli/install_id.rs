use std::path::PathBuf;

use clap::Parser;

use crate::patcher::change_install_id;

/// Change dylib id. Works only if your object is a shared library.
#[derive(Parser, Debug)]
pub struct Args {
    /// Path to the file to remove
    #[arg(short)]
    pub path: PathBuf,

    /// New rpath to add
    #[arg(short)]
    pub new_install_id: String,
}

pub fn execute(args: Args) {
    let bytes_of_file = std::fs::read(&args.path).unwrap();

    let changed_buffer = change_install_id(bytes_of_file, args.new_install_id);

    let new_path = args.path.with_file_name(format!(
        "{}_changed_install_id",
        args.path.file_name().unwrap().to_str().unwrap()
    ));

    std::fs::write(new_path, changed_buffer).unwrap();
}
