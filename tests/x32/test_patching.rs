use arwen::patcher::{
    add_rpath, change_install_id, change_install_name, change_rpath, remove_rpath,
};
use goblin::mach::MachO;
use std::path::PathBuf;

/// This test checks if the rpath of a Mach-O file can be changed.
#[test]
fn test_rpath_change() {
    // load hello_with_rpath from data
    let package_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data = package_dir.join("tests/data/x32/ls");

    let data_bytes = std::fs::read(&data).unwrap();

    // we try to change the rpath of the file to something longer
    let changed = change_rpath(
        data_bytes,
        "path_graf".to_string(),
        "path_graf_path_graf_path_graf_path_graf".to_string(),
    );

    let changed_macho = MachO::parse(&changed, 0).unwrap();

    insta::assert_debug_snapshot!(changed_macho);
    insta::assert_debug_snapshot!(changed_macho.rpaths);

    insta::assert_snapshot!(changed.len(), @"49120");
}

/// This test checks if the rpath of a Mach-O file can be removed.
#[test]
fn test_rpath_remove() {
    // load hello_with_rpath from data
    let package_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data = package_dir.join("tests/data/hello_with_rpath");

    let data_bytes = std::fs::read(&data).unwrap();

    // we try to change the rpath of the file to something longer
    let changed = remove_rpath(data_bytes, "path_graf".to_string());

    let changed_macho = MachO::parse(&changed, 0).unwrap();

    insta::assert_debug_snapshot!(changed_macho);
    insta::assert_debug_snapshot!(changed_macho.rpaths);
    insta::assert_snapshot!(changed.len(), @"33440");
}

/// This test verify that a rpath can be added to a Mach-O file.
#[test]
fn test_rpath_add() {
    // load hello_with_rpath from data
    let package_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data = package_dir.join("tests/data/hello_with_rpath");

    let data_bytes = std::fs::read(&data).unwrap();

    // we try to change the rpath of the file to something longer
    let changed = add_rpath(
        data_bytes,
        "abababababababababababaabbababababababababab".to_string(),
    );

    let changed_macho = MachO::parse(&changed, 0).unwrap();

    insta::assert_debug_snapshot!(changed_macho);
    insta::assert_debug_snapshot!(changed_macho.rpaths);
    insta::assert_snapshot!(changed.len(), @"33440");
}

/// This test verify that an dylib id can be changed in a Mach-O file.
#[test]
fn test_change_dylib_id() {
    // load hello_with_rpath from data
    let package_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data = package_dir.join("tests/data/libmylib.dylib");

    let data_bytes = std::fs::read(&data).unwrap();

    // we try to change the rpath of the file to something longer
    let changed = change_install_id(
        data_bytes,
        "very_very_very_very_very_very_very_longid.dylib".to_string(),
    );

    let changed_macho = MachO::parse(&changed, 0).unwrap();

    insta::assert_debug_snapshot!(changed_macho);
    insta::assert_debug_snapshot!(changed_macho.name);
    insta::assert_snapshot!(changed.len(), @"33355");
}

/// This test verify that a name of dynamic library can be changed
#[test]
fn test_change_dylib_name() {
    // load hello_with_rpath from data
    let package_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data = package_dir.join("tests/data/hello_with_rpath");

    let data_bytes = std::fs::read(&data).unwrap();

    // we try to change the rpath of the file to something longer
    let changed = change_install_name(
        data_bytes,
        "/usr/lib/libSystem.B.dylib".to_string(),
        "very_very_very_very_very_very_very_longid".to_string(),
    );

    let changed_macho = MachO::parse(&changed, 0).unwrap();

    insta::assert_debug_snapshot!(changed_macho);
    insta::assert_debug_snapshot!(changed_macho.libs);
    insta::assert_snapshot!(changed.len(), @"33440");
}
