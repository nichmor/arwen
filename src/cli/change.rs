use std::path::PathBuf;

use clap::Parser;

use crate::patcher::change_rpath;

/// Change already existing run path
#[derive(Parser, Debug)]
pub struct Args {
    /// Path to the file to remove
    #[arg(short)]
    pub path: PathBuf,

    /// Old rpath to remove
    #[arg(short)]
    pub old_rpath: String,

    /// New rpath to add
    #[arg(short)]
    pub new_rpath: String,
}

pub fn execute(args: Args) {
    let bytes_of_file = std::fs::read(&args.path).unwrap();

    let changed_buffer = change_rpath(bytes_of_file, args.old_rpath, args.new_rpath);

    let new_path = args.path.with_file_name(format!(
        "{}_changed_rpath",
        args.path.file_name().unwrap().to_str().unwrap()
    ));

    std::fs::write(new_path, changed_buffer).unwrap();
}
