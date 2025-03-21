use std::process::{Command, Output};

pub fn run_command(cmd: &str, args: &[&str]) -> std::io::Result<Output> {
    let output = Command::new(cmd).args(args).output()?;

    if !output.status.success() {
        eprintln!("Command {:?} failed", cmd);
    }
    Ok(output)
}

#[cfg(target_os = "macos")]
pub(crate) fn codesign_binary(base_binary: &str) -> std::io::Result<()> {
    run_command("codesign", &["--force", "--sign", "-", base_binary]).unwrap();
    Ok(())
}

#[cfg(target_os = "macos")]
pub(crate) fn calculate_md5_hash(base_binary: &str) -> std::io::Result<String> {
    let md5_output = Command::new("md5").arg(base_binary).output().unwrap();
    let md5_hash = String::from_utf8_lossy(&md5_output.stdout)
        .split_whitespace()
        .last()
        .unwrap_or("")
        .to_string();
    Ok(md5_hash)
}

// fn run_tool_command(tool: &Tool, args: &[&str]) -> std::io::Result<()> {
//     match tool {
//         Tool::InstallNameTool => {
//             run_command("install_name_tool", args).unwrap();
//         }
//         Tool::Arwen => {
//             run_command("arwen", args).unwrap();
//         }
//     }
//     Ok(())
// }

pub enum Tool {
    InstallNameTool,
    Arwen,
}
