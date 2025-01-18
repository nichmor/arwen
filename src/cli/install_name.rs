use std::path::PathBuf;

use clap::Parser;

use crate::patcher::change_install_name;

/// Change existing dylib load name
#[derive(Parser, Debug)]
pub struct Args {
    /// Path to the file to remove
    #[arg(short)]
    pub path: PathBuf,

    /// Old rpath to remove
    #[arg(short)]
    pub old_install_name: String,

    /// New rpath to add
    #[arg(short)]
    pub new_install_name: String,
}

pub fn execute(args: Args) {
    let bytes_of_file = std::fs::read(&args.path).unwrap();

    let changed_buffer =
        change_install_name(bytes_of_file, args.old_install_name, args.new_install_name);

    let new_path = args.path.with_file_name(format!(
        "{}_changed_install_name",
        args.path.file_name().unwrap().to_str().unwrap()
    ));

    std::fs::write(new_path, changed_buffer).unwrap();
}
