use std::process::Command;

pub fn run_command(cmd: &str, args: &[&str]) -> std::io::Result<()> {
    let status = Command::new(cmd).args(args).status()?;

    if !status.success() {
        eprintln!("Command {:?} failed", cmd);
    }
    Ok(())
}
