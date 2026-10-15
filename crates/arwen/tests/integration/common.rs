use std::process::{Command, Output};

/// The `arwen` binary built by Cargo for this test run.
pub const ARWEN: &str = env!("CARGO_BIN_EXE_arwen");

pub fn run_command(cmd: &str, args: &[&str]) -> std::io::Result<Output> {
    let output = Command::new(cmd).args(args).output()?;

    if !output.status.success() {
        eprintln!("Command {cmd:?} failed");
    }
    Ok(output)
}
