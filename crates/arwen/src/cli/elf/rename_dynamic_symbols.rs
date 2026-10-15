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

/// Renames dynamic symbols
#[derive(Parser, Debug)]
pub struct Args {
    /// Path to the file to change
    pub path_to_binary: PathBuf,

    // Dynamic symbols to rename
    #[arg(value_parser = parse_key_val)]
    pub rename_symbols: Vec<(String, String)>,
}

pub fn execute(args: Args) -> Result<(), arwen_elf::ElfError> {
    super::patch(&args.path_to_binary, |elf| {
        let mut rename_symbols = HashMap::new();
        for (key, value) in args.rename_symbols {
            rename_symbols.insert(key, value);
        }

        elf.rename_dynamic_symbols(&rename_symbols)?;
        Ok(())
    })
}
