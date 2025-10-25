use arwen::cli::execute;
use std::process::ExitCode;

fn main() -> ExitCode {
    if let Err(e) = execute() {
        eprintln!("{e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
