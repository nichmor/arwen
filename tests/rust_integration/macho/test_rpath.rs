use std::fs::{self, create_dir_all};
use std::path::PathBuf;

use rstest::rstest;
use tempfile::tempdir;

#[cfg(not(target_os = "macos"))]
use crate::macho::common::run_command;

#[cfg(target_os = "macos")]
use crate::macho::common::{calculate_md5_hash, codesign_binary, run_command};

use super::common::Tool;

fn add_rpath(base_binary: &str, tool: &Tool) -> std::io::Result<()> {
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
    };

    Ok(())
}

fn remove_rpath(base_binary: &str, tool: &Tool) -> std::io::Result<()> {
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
    };

    Ok(())
}

fn change_rpath(base_binary: &str, tool: &Tool) -> std::io::Result<()> {
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
    };
    Ok(())
}

fn change_install_name(base_binary: &str, tool: &Tool) -> std::io::Result<()> {
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
    };

    Ok(())
}

#[rstest]
fn test_add_rpath(#[files("tests/data/macho/*/exec/*")] bin_path: PathBuf) {
    let temp_folder = tempdir().unwrap().path().join("test_add_rpath");

    // let temp_folder = std::env::current_dir().unwrap();
    fs::create_dir_all(&temp_folder).unwrap();

    let base_install_name_tool_binary = temp_folder.join("install_name_tool/hello_with_rpath.bin");

    let base_arwen_binary = temp_folder.join("arwen/hello_with_rpath.bin");

    create_dir_all(temp_folder.join("arwen")).unwrap();
    create_dir_all(temp_folder.join("install_name_tool")).unwrap();

    fs::copy(&bin_path, &base_install_name_tool_binary).unwrap();
    fs::copy(&bin_path, &base_arwen_binary).unwrap();

    // add the rpath
    add_rpath(
        base_install_name_tool_binary.to_str().unwrap(),
        &Tool::InstallNameTool,
    )
    .unwrap();
    add_rpath(base_arwen_binary.to_str().unwrap(), &Tool::Arwen).unwrap();

    // read with llvm-otool the load command
    let rpath_load_command =
        run_command("llvm-otool", &["-l", base_arwen_binary.to_str().unwrap()]).unwrap();

    let expected_rpath = r#"path path_graf (offset 12)"#;

    assert!(String::from_utf8(rpath_load_command.stdout)
        .unwrap()
        .contains(expected_rpath));

    #[cfg(target_os = "macos")]
    {
        codesign_binary(base_install_name_tool_binary.to_str().unwrap()).unwrap();
        codesign_binary(base_arwen_binary.to_str().unwrap()).unwrap();

        let hash1 = calculate_md5_hash(base_install_name_tool_binary.to_str().unwrap()).unwrap();
        let hash2 = calculate_md5_hash(base_arwen_binary.to_str().unwrap()).unwrap();

        assert_eq!(hash1, hash2);
    }
}

#[rstest]
fn test_remove_rpath(#[files("tests/data/macho/*/exec/*")] bin_path: PathBuf) {
    let temp_folder = tempdir().unwrap().path().join("test_remove_rpath");
    fs::create_dir_all(&temp_folder).unwrap();

    let base_install_name_tool_binary = temp_folder.join("install_name_tool/hello_with_rpath.bin");

    let base_arwen_binary = temp_folder.join("arwen/hello_with_rpath.bin");

    create_dir_all(temp_folder.join("arwen")).unwrap();
    create_dir_all(temp_folder.join("install_name_tool")).unwrap();

    fs::copy(&bin_path, &base_install_name_tool_binary).unwrap();
    fs::copy(&bin_path, &base_arwen_binary).unwrap();

    // remove rpath
    remove_rpath(
        base_install_name_tool_binary.to_str().unwrap(),
        &Tool::InstallNameTool,
    )
    .unwrap();
    remove_rpath(base_arwen_binary.to_str().unwrap(), &Tool::Arwen).unwrap();

    // read with llvm-otool the load command
    let rpath_load_command =
        run_command("llvm-otool", &["-l", base_arwen_binary.to_str().unwrap()]).unwrap();

    let expected_absent_rpath = r#"path_graf"#;

    assert!(!String::from_utf8(rpath_load_command.stdout)
        .unwrap()
        .contains(expected_absent_rpath));

    #[cfg(target_os = "macos")]
    {
        codesign_binary(base_install_name_tool_binary.to_str().unwrap()).unwrap();
        codesign_binary(base_arwen_binary.to_str().unwrap()).unwrap();

        let hash1 = calculate_md5_hash(base_install_name_tool_binary.to_str().unwrap()).unwrap();

        let hash2 = calculate_md5_hash(base_arwen_binary.to_str().unwrap()).unwrap();
        assert_eq!(hash1, hash2);
    }
}

#[rstest]
fn test_change_rpath(#[files("tests/data/macho/*/exec/*")] bin_path: PathBuf) {
    let temp_folder = tempdir().unwrap().path().join("test_change_rpath");
    fs::create_dir_all(&temp_folder).unwrap();

    let base_install_name_tool_binary = temp_folder.join("install_name_tool/hello_with_rpath.bin");

    let base_arwen_binary = temp_folder.join("arwen/hello_with_rpath.bin");

    create_dir_all(temp_folder.join("arwen")).unwrap();
    create_dir_all(temp_folder.join("install_name_tool")).unwrap();

    fs::copy(&bin_path, &base_install_name_tool_binary).unwrap();
    fs::copy(&bin_path, &base_arwen_binary).unwrap();

    // change the rpath
    change_rpath(
        base_install_name_tool_binary.to_str().unwrap(),
        &Tool::InstallNameTool,
    )
    .unwrap();
    change_rpath(base_arwen_binary.to_str().unwrap(), &Tool::Arwen).unwrap();

    // read with llvm-otool the load command
    let rpath_load_command =
        run_command("llvm-otool", &["-l", base_arwen_binary.to_str().unwrap()]).unwrap();

    let expected_rpath = r#"path test_path (offset 12)"#;

    assert!(String::from_utf8(rpath_load_command.stdout)
        .unwrap()
        .contains(expected_rpath));

    #[cfg(target_os = "macos")]
    {
        codesign_binary(base_install_name_tool_binary.to_str().unwrap()).unwrap();
        codesign_binary(base_arwen_binary.to_str().unwrap()).unwrap();

        let hash1 = calculate_md5_hash(base_install_name_tool_binary.to_str().unwrap()).unwrap();
        let hash2 = calculate_md5_hash(base_arwen_binary.to_str().unwrap()).unwrap();

        assert_eq!(hash1, hash2);
    }
}

#[rstest]
fn test_change_install_name(#[files("tests/data/macho/*/exec/*")] bin_path: PathBuf) {
    let temp_folder = tempdir().unwrap().path().join("test_change_install_name");
    fs::create_dir_all(&temp_folder).unwrap();

    let base_install_name_tool_binary = temp_folder.join("install_name_tool/hello_with_rpath.bin");

    let base_arwen_binary = temp_folder.join("arwen/hello_with_rpath.bin");

    create_dir_all(temp_folder.join("arwen")).unwrap();
    create_dir_all(temp_folder.join("install_name_tool")).unwrap();

    fs::copy(&bin_path, &base_install_name_tool_binary).unwrap();
    fs::copy(&bin_path, &base_arwen_binary).unwrap();

    // change the install name
    change_install_name(
        base_install_name_tool_binary.to_str().unwrap(),
        &Tool::InstallNameTool,
    )
    .unwrap();

    change_install_name(base_arwen_binary.to_str().unwrap(), &Tool::Arwen).unwrap();

    // read with llvm-otool the load command
    let name_of_dylib_used =
        run_command("llvm-otool", &["-L", base_arwen_binary.to_str().unwrap()]).unwrap();

    let expected_dylib_name = r#"new_lib_system.id (compatibility version 1.0.0"#;

    assert!(String::from_utf8(name_of_dylib_used.stdout)
        .unwrap()
        .contains(expected_dylib_name));

    #[cfg(target_os = "macos")]
    {
        codesign_binary(base_install_name_tool_binary.to_str().unwrap()).unwrap();
        codesign_binary(base_arwen_binary.to_str().unwrap()).unwrap();

        let hash1 = calculate_md5_hash(base_install_name_tool_binary.to_str().unwrap()).unwrap();
        let hash2 = calculate_md5_hash(base_arwen_binary.to_str().unwrap()).unwrap();

        assert_eq!(hash1, hash2);
    }
}
