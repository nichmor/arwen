use std::fs::{self, create_dir_all};
use std::path::PathBuf;
use std::process::Command;

use tempfile::tempdir;

use crate::common::run_command;

pub enum Tool {
    InstallNameTool,
    Arwen,
}

fn add_rpath_and_sign(base_binary: &str, tool: &Tool) -> std::io::Result<String> {
    match tool {
        Tool::InstallNameTool => {
            run_command(
                "install_name_tool",
                &["-add_rpath", "new_graf", base_binary],
            )
            .unwrap();
        }
        Tool::Arwen => {
            run_command("arwen", &["macho", "add-rpath", "new_graf", base_binary]).unwrap();
        }
    }

    run_command("codesign", &["--force", "--sign", "-", base_binary]).unwrap();

    let md5_output = Command::new("md5").arg(base_binary).output().unwrap();

    let md5_hash = String::from_utf8_lossy(&md5_output.stdout)
        .split_whitespace()
        .last()
        .unwrap_or("")
        .to_string();
    Ok(md5_hash)
}

fn remove_rpath_and_sign(base_binary: &str, tool: &Tool) -> std::io::Result<String> {
    match tool {
        Tool::InstallNameTool => {
            run_command(
                "install_name_tool",
                &["-delete_rpath", "path_graf", base_binary],
            )
            .unwrap();
        }
        Tool::Arwen => {
            run_command(
                "arwen",
                &["macho", "delete-rpath", "path_graf", base_binary],
            )
            .unwrap();
        }
    }

    run_command("codesign", &["--force", "--sign", "-", base_binary]).unwrap();

    let md5_output = Command::new("md5").arg(base_binary).output().unwrap();

    let md5_hash = String::from_utf8_lossy(&md5_output.stdout)
        .split_whitespace()
        .last()
        .unwrap_or("")
        .to_string();
    Ok(md5_hash)
}

fn change_rpath_and_codesign(base_binary: &str, tool: &Tool) -> std::io::Result<String> {
    match tool {
        Tool::InstallNameTool => {
            run_command(
                "install_name_tool",
                &["-rpath", "path_graf", "test_path", base_binary],
            )
            .unwrap();
        }
        Tool::Arwen => {
            run_command(
                "arwen",
                &[
                    "macho",
                    "change-rpath",
                    "path_graf",
                    "test_path",
                    base_binary,
                ],
            )
            .unwrap();
        }
    }

    run_command("codesign", &["--force", "--sign", "-", base_binary]).unwrap();

    let md5_output = Command::new("md5").arg(base_binary).output().unwrap();

    let md5_hash = String::from_utf8_lossy(&md5_output.stdout)
        .split_whitespace()
        .last()
        .unwrap_or("")
        .to_string();
    Ok(md5_hash)
}

fn change_install_name_and_codesign(base_binary: &str, tool: &Tool) -> std::io::Result<String> {
    match tool {
        Tool::InstallNameTool => {
            run_command(
                "install_name_tool",
                &[
                    "-change",
                    "/usr/lib/libSystem.B.dylib",
                    "new_lib_system.id",
                    base_binary,
                ],
            )
            .unwrap();
        }
        Tool::Arwen => {
            run_command(
                "arwen",
                &[
                    "macho",
                    "change-install-name",
                    "/usr/lib/libSystem.B.dylib",
                    "new_lib_system.id",
                    base_binary,
                ],
            )
            .unwrap();
        }
    }

    run_command("codesign", &["--force", "--sign", "-", base_binary]).unwrap();

    let md5_output = Command::new("md5").arg(base_binary).output().unwrap();

    let md5_hash = String::from_utf8_lossy(&md5_output.stdout)
        .split_whitespace()
        .last()
        .unwrap_or("")
        .to_string();
    Ok(md5_hash)
}

#[test]
fn test_add_rpath() {
    let package_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data = package_dir.join("tests/data/hello_with_rpath");

    let temp_folder = tempdir().unwrap().path().join("test_add_rpath");
    fs::create_dir_all(&temp_folder).unwrap();

    let base_install_name_tool_binary = temp_folder.join("install_name_tool/hello_with_rpath.bin");

    let base_arwen_binary = temp_folder.join("arwen/hello_with_rpath.bin");

    create_dir_all(temp_folder.join("arwen")).unwrap();
    create_dir_all(temp_folder.join("install_name_tool")).unwrap();

    fs::copy(&data, &base_install_name_tool_binary).unwrap();
    fs::copy(&data, &base_arwen_binary).unwrap();

    let hash1 = add_rpath_and_sign(
        base_install_name_tool_binary.to_str().unwrap(),
        &Tool::InstallNameTool,
    )
    .unwrap();
    let hash2 = add_rpath_and_sign(base_arwen_binary.to_str().unwrap(), &Tool::Arwen).unwrap();

    assert_eq!(hash1, hash2);
}

#[test]
fn test_remove_rpath() {
    let package_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data = package_dir.join("tests/data/hello_with_rpath");

    let temp_folder = tempdir().unwrap().path().join("test_remove_rpath");
    fs::create_dir_all(&temp_folder).unwrap();

    let base_install_name_tool_binary = temp_folder.join("install_name_tool/hello_with_rpath.bin");

    let base_arwen_binary = temp_folder.join("arwen/hello_with_rpath.bin");

    create_dir_all(temp_folder.join("arwen")).unwrap();
    create_dir_all(temp_folder.join("install_name_tool")).unwrap();

    fs::copy(&data, &base_install_name_tool_binary).unwrap();
    fs::copy(&data, &base_arwen_binary).unwrap();

    let hash1 = remove_rpath_and_sign(
        base_install_name_tool_binary.to_str().unwrap(),
        &Tool::InstallNameTool,
    )
    .unwrap();
    let hash2 = remove_rpath_and_sign(base_arwen_binary.to_str().unwrap(), &Tool::Arwen).unwrap();

    assert_eq!(hash1, hash2);
}

#[test]
fn test_change_rpath() {
    let package_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data = package_dir.join("tests/data/hello_with_rpath");

    let temp_folder = tempdir().unwrap().path().join("test_change_rpath");
    fs::create_dir_all(&temp_folder).unwrap();

    let base_install_name_tool_binary = temp_folder.join("install_name_tool/hello_with_rpath.bin");

    let base_arwen_binary = temp_folder.join("arwen/hello_with_rpath.bin");

    create_dir_all(temp_folder.join("arwen")).unwrap();
    create_dir_all(temp_folder.join("install_name_tool")).unwrap();

    fs::copy(&data, &base_install_name_tool_binary).unwrap();
    fs::copy(&data, &base_arwen_binary).unwrap();

    let hash1 = change_rpath_and_codesign(
        base_install_name_tool_binary.to_str().unwrap(),
        &Tool::InstallNameTool,
    )
    .unwrap();
    let hash2 =
        change_rpath_and_codesign(base_arwen_binary.to_str().unwrap(), &Tool::Arwen).unwrap();

    assert_eq!(hash1, hash2);
}

#[test]
fn test_change_install_name() {
    let package_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data = package_dir.join("tests/data/hello_with_rpath");

    let temp_folder = tempdir().unwrap().path().join("test_change_install_name");
    fs::create_dir_all(&temp_folder).unwrap();

    let base_install_name_tool_binary = temp_folder.join("install_name_tool/hello_with_rpath.bin");

    let base_arwen_binary = temp_folder.join("arwen/hello_with_rpath.bin");

    create_dir_all(temp_folder.join("arwen")).unwrap();
    create_dir_all(temp_folder.join("install_name_tool")).unwrap();

    fs::copy(&data, &base_install_name_tool_binary).unwrap();
    fs::copy(&data, &base_arwen_binary).unwrap();

    let hash1 = change_install_name_and_codesign(
        base_install_name_tool_binary.to_str().unwrap(),
        &Tool::InstallNameTool,
    )
    .unwrap();
    let hash2 = change_install_name_and_codesign(base_arwen_binary.to_str().unwrap(), &Tool::Arwen)
        .unwrap();

    assert_eq!(hash1, hash2);
}
