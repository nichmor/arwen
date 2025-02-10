use arwen::macho::MachoContainer;
use goblin::mach::{MultiArch, SingleArch};
use std::path::PathBuf;

/// This test checks if the rpath of a Mach-O file can be changed.
///

const FAT_BINARY_PATH: &str = "tests/data/fat/i386_x86_64/hello_with_some_rpath.bin";
const FAT_LIB_PATH: &str = "tests/data/fat/x64_x86_64/fatlib";

#[test]
fn test_rpath_change() {
    // load hello_with_rpath from data
    let package_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data = package_dir.join(FAT_BINARY_PATH);

    let data_bytes = std::fs::read(&data).unwrap();

    let mut macho_container = MachoContainer::parse(&data_bytes).unwrap();

    macho_container
        .change_rpath("path_graf", "path_graf_path_graf_path_graf_path_graf")
        .unwrap();

    let fat_macho = MultiArch::new(&macho_container.data).unwrap();

    for arch in fat_macho.into_iter() {
        let single_arch = arch.unwrap();

        let SingleArch::MachO(changed_macho) = single_arch else {
            panic!("Expected MachO, got {:?}", single_arch);
        };

        // let changed_macho = MachO::parse(&arch.data, 0).unwrap();
        insta::assert_debug_snapshot!(changed_macho);
        insta::assert_debug_snapshot!(changed_macho.rpaths);
    }

    insta::assert_snapshot!(macho_container.data.len(), @"76224");
}

/// This test checks if the rpath of a Mach-O file can be removed.
#[test]
fn test_rpath_remove() {
    // load hello_with_rpath from data
    let package_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data = package_dir.join(FAT_BINARY_PATH);

    let data_bytes = std::fs::read(&data).unwrap();

    let mut macho_container = MachoContainer::parse(&data_bytes).unwrap();

    // we try to change the rpath of the file to something longer
    macho_container.remove_rpath("path_graf").unwrap();

    let fat_macho = MultiArch::new(&macho_container.data).unwrap();

    for arch in fat_macho.into_iter() {
        let single_arch = arch.unwrap();

        let SingleArch::MachO(changed_macho) = single_arch else {
            panic!("Expected MachO, got {:?}", single_arch);
        };

        // let changed_macho = MachO::parse(&arch.data, 0).unwrap();
        insta::assert_debug_snapshot!(changed_macho);
        insta::assert_debug_snapshot!(changed_macho.rpaths);
    }

    insta::assert_snapshot!(macho_container.data.len(), @"76224");
}

/// This test verify that a rpath can be added to a Mach-O file.
#[test]
fn test_rpath_add() {
    // load hello_with_rpath from data
    let package_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data = package_dir.join(FAT_BINARY_PATH);

    let data_bytes = std::fs::read(&data).unwrap();

    let mut macho_container = MachoContainer::parse(&data_bytes).unwrap();

    // we try to change the rpath of the file to something longer
    macho_container
        .add_rpath("abababababababababababaabbababababababababab")
        .unwrap();

    let fat_macho = MultiArch::new(&macho_container.data).unwrap();

    for arch in fat_macho.into_iter() {
        let single_arch = arch.unwrap();

        let SingleArch::MachO(changed_macho) = single_arch else {
            panic!("Expected MachO, got {:?}", single_arch);
        };

        // let changed_macho = MachO::parse(&arch.data, 0).unwrap();
        insta::assert_debug_snapshot!(changed_macho);
        insta::assert_debug_snapshot!(changed_macho.rpaths);
    }

    insta::assert_snapshot!(macho_container.data.len(), @"76224");
}

/// This test verify that an dylib id can be changed in a Mach-O file.
#[test]
fn test_change_dylib_id() {
    // load hello_with_rpath from data
    let package_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data = package_dir.join(FAT_LIB_PATH);

    let data_bytes = std::fs::read(&data).unwrap();

    let mut macho_container = MachoContainer::parse(&data_bytes).unwrap();

    // we try to change the rpath of the file to something longer
    macho_container
        .change_install_id("very_very_very_very_very_very_very_longid.dylib")
        .unwrap();

    let fat_macho = MultiArch::new(&macho_container.data).unwrap();

    for arch in fat_macho.into_iter() {
        let single_arch = arch.unwrap();

        let SingleArch::MachO(changed_macho) = single_arch else {
            panic!("Expected MachO, got {:?}", single_arch);
        };

        // let changed_macho = MachO::parse(&arch.data, 0).unwrap();
        insta::assert_debug_snapshot!(changed_macho);
        insta::assert_debug_snapshot!(changed_macho.name);
    }

    insta::assert_snapshot!(macho_container.data.len(), @"98891");
}

/// This test verify that a name of dynamic library can be changed
#[test]
fn test_change_dylib_name() {
    // load hello_with_rpath from data
    let package_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data = package_dir.join(FAT_BINARY_PATH);

    let data_bytes = std::fs::read(&data).unwrap();

    let mut macho_container = MachoContainer::parse(&data_bytes).unwrap();

    // we try to change the rpath of the file to something longer
    macho_container
        .change_install_name(
            "/usr/lib/libSystem.B.dylib",
            "very_very_very_very_very_very_very_longid",
        )
        .unwrap();

    let fat_macho = MultiArch::new(&macho_container.data).unwrap();

    for arch in fat_macho.into_iter() {
        let single_arch = arch.unwrap();

        let SingleArch::MachO(changed_macho) = single_arch else {
            panic!("Expected MachO, got {:?}", single_arch);
        };

        // let changed_macho = MachO::parse(&arch.data, 0).unwrap();
        insta::assert_debug_snapshot!(changed_macho);
        insta::assert_debug_snapshot!(changed_macho.libs);
    }

    insta::assert_snapshot!(macho_container.data.len(), @"76224");
}
