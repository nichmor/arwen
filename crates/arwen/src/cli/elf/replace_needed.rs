use std::{collections::HashMap, error::Error, path::PathBuf};

use clap::Parser;

/// Parse a single key-value pair
fn parse_key_val(s: &str) -> Result<(String, String), Box<dyn Error + Send + Sync + 'static>> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    let key = s[..pos].to_string();
    let value = s[pos + 1..].to_string();
    Ok((key, value))
}

/// Replace dependencies from DT_NEEDED
#[derive(Parser, Debug)]
pub struct Args {
    /// Path to the file to change
    pub path_to_binary: PathBuf,

    // DT_NEEDED to replace
    #[arg(value_parser = parse_key_val)]
    pub dt_needed: Vec<(String, String)>,
}

pub fn execute(args: Args) -> Result<(), arwen_elf::ElfError> {
    let bytes_of_file = std::fs::read(&args.path_to_binary).unwrap();

    let mut elf = arwen_elf::ElfContainer::parse(&bytes_of_file)?;

    let mut dt_needed = HashMap::new();
    for (key, value) in args.dt_needed {
        dt_needed.insert(key, value);
    }

    elf.replace_needed(&dt_needed)?;

    let output_file =
        std::fs::File::create(format!("{}", args.path_to_binary.to_string_lossy())).unwrap();

    elf.write(&output_file)?;

    Ok(())
}
