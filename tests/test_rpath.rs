use goblin::mach::MachO;
use std::path::PathBuf;

use arwen::macho::MachoContainer;

/// This test checks if the rpath of a Mach-O file can be changed.
#[test]
fn test_rpath_change() {
    // load hello_with_rpath from data
    let package_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data = package_dir.join("tests/data/hello_with_rpath");

    let data_bytes = std::fs::read(&data).unwrap();

    let mut macho_container = MachoContainer::parse(&data_bytes).unwrap();

    // we try to change the rpath of the file to something longer
    macho_container
        .change_rpath("path_graf", "path_graf_path_graf_path_graf_path_graf")
        .unwrap();

    let changed_macho = goblin::mach::MachO::parse(&macho_container.data, 0).unwrap();

    insta::assert_debug_snapshot!(changed_macho);
    insta::assert_debug_snapshot!(changed_macho.rpaths);

    insta::assert_snapshot!(macho_container.data.len(), @"33440");
}

/// This test checks if the rpath of a Mach-O file can be removed.
#[test]
fn test_rpath_remove() {
    // load hello_with_rpath from data
    let package_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data = package_dir.join("tests/data/hello_with_rpath");

    let data_bytes = std::fs::read(&data).unwrap();

    // we try to change the rpath of the file to something longer
    let mut macho_container = MachoContainer::parse(&data_bytes).unwrap();

    // we try to change the rpath of the file to something longer
    macho_container.remove_rpath("path_graf").unwrap();

    let changed_macho = MachO::parse(&macho_container.data, 0).unwrap();

    insta::assert_debug_snapshot!(changed_macho);
    insta::assert_debug_snapshot!(changed_macho.rpaths);
    insta::assert_snapshot!(macho_container.data.len(), @"33440");
}

/// This test verify that a rpath can be added to a Mach-O file.
#[test]
fn test_rpath_add() {
    // load hello_with_rpath from data
    let package_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data = package_dir.join("tests/data/hello_with_rpath");

    let data_bytes = std::fs::read(&data).unwrap();

    // we try to change the rpath of the file to something longer
    let mut macho_container = MachoContainer::parse(&data_bytes).unwrap();

    macho_container
        .add_rpath("abababababababababababaabbababababababababab")
        .unwrap();

    let changed_macho = MachO::parse(&macho_container.data, 0).unwrap();

    insta::assert_debug_snapshot!(changed_macho);
    insta::assert_debug_snapshot!(changed_macho.rpaths);
    insta::assert_snapshot!(macho_container.data.len(), @"33440");
}

/// This test verify that an dylib id can be changed in a Mach-O file.
#[test]
fn test_change_dylib_id() {
    // load hello_with_rpath from data
    let package_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data = package_dir.join("tests/data/libmylib.dylib");

    let data_bytes = std::fs::read(&data).unwrap();

    let mut macho_container = MachoContainer::parse(&data_bytes).unwrap();

    // we try to change the install id of the dlib to something longer
    macho_container
        .change_install_id("very_very_very_very_very_very_very_longid.dylib")
        .unwrap();

    let changed_macho = MachO::parse(&macho_container.data, 0).unwrap();

    insta::assert_debug_snapshot!(changed_macho);
    insta::assert_debug_snapshot!(changed_macho.name);
    insta::assert_snapshot!(macho_container.data.len(), @"33355");
}

/// This test verify that a name of dynamic library can be changed
#[test]
fn test_change_dylib_name() {
    // load hello_with_rpath from data
    let package_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data = package_dir.join("tests/data/hello_with_rpath");

    let data_bytes = std::fs::read(&data).unwrap();

    let mut macho_container = MachoContainer::parse(&data_bytes).unwrap();

    // we try to change the rpath of the file to something longer
    macho_container
        .change_install_name(
            "/usr/lib/libSystem.B.dylib",
            "very_very_very_very_very_very_very_longid",
        )
        .unwrap();

    let changed_macho = MachO::parse(&macho_container.data, 0).unwrap();

    insta::assert_debug_snapshot!(changed_macho);
    insta::assert_debug_snapshot!(changed_macho.libs);
    insta::assert_snapshot!(macho_container.data.len(), @"33440");
}
