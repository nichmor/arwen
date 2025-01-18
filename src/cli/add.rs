use std::path::PathBuf;

use clap::Parser;

use crate::patcher::add_rpath;

/// Add a run path
#[derive(Parser, Debug)]
pub struct Args {
    /// Path to the file to remove
    #[arg(short)]
    pub path: PathBuf,

    /// New rpath to add
    #[arg(short)]
    pub new_rpath: String,
}

pub fn execute(args: Args) {
    let bytes_of_file = std::fs::read(&args.path).unwrap();

    let changed_buffer = add_rpath(bytes_of_file, args.new_rpath);

    let new_path = args.path.with_file_name(format!(
        "{}_added_rpath",
        args.path.file_name().unwrap().to_str().unwrap()
    ));

    std::fs::write(new_path, changed_buffer).unwrap();
}
