//! CLI tool for ad-hoc code signing using arwen-codesign
//!
//! This tool provides a simple interface for signing Mach-O binaries,
//! compatible with the test suite expectations.

use arwen_codesign::{adhoc_sign, AdhocSignOptions, Entitlements};
use clap::Parser;
use std::path::PathBuf;
use std::process;
use std::fs;

#[derive(Parser)]
#[command(name = "codesign_tool")]
#[command(about = "Ad-hoc code signing for Mach-O binaries", long_about = None)]
struct Args {
    /// Binary file to sign
    file: PathBuf,

    /// Identifier for the signature
    #[arg(short, long)]
    identifier: String,

    /// Enable hardened runtime (equivalent to --options runtime)
    #[arg(long = "hardened-runtime")]
    hardened_runtime: bool,

    /// Preserve existing entitlements
    #[arg(long = "preserve-entitlements")]
    preserve_entitlements: bool,

    /// Set linker-signed flag
    #[arg(long = "linker-signed")]
    linker_signed: bool,
}

fn main() {
    let args = Args::parse();

    // Validate file exists
    if !args.file.exists() {
        eprintln!("Error: File not found: {}", args.file.display());
        process::exit(1);
    }

    // Build signing options
    let mut options = AdhocSignOptions::new(&args.identifier);

    if args.hardened_runtime {
        options = options.with_hardened_runtime();
    }

    if args.linker_signed {
        options = options.with_linker_signed();
    }

    if args.preserve_entitlements {
        options = options.with_entitlements(Entitlements::Preserve);
    }

    // Perform signing
    // Read the binary file
    let data = match fs::read(&args.file) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error reading {}: {}", args.file.display(), e);
            process::exit(1);
        }
    };

    // Sign it
    let signed_data = match adhoc_sign(data, &options) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error signing {}: {}", args.file.display(), e);
            process::exit(1);
        }
    };

    // Write the signed binary back
    match fs::write(&args.file, signed_data) {
        Ok(()) => {
            println!("Successfully signed: {}", args.file.display());
        }
        Err(e) => {
            eprintln!("Error writing {}: {}", args.file.display(), e);
            process::exit(1);
        }
    }
}
