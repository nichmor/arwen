/// This test checks if the rpath of a Mach-O file can be changed.
use arwen::macho::MachoContainer;
use goblin::mach::{MultiArch, SingleArch};
use rstest::rstest;
use std::path::PathBuf;

// const FAT_BINARY_PATH: &str = "tests/data/macho/fat/i386_x86_64/hello_with_some_rpath.bin";
// const FAT_LIB_PATH: &str = "tests/data/macho/fat/x64_x86_64/fatlib";

#[rstest]
fn test_rpath_change(#[files("tests/data/macho/fat/exec/*")] bin_path: PathBuf) {
    // load hello_with_rpath from data

    let data_bytes = std::fs::read(&bin_path).unwrap();

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
#[rstest]
fn test_rpath_remove(#[files("tests/data/macho/fat/exec/*")] bin_path: PathBuf) {
    let data_bytes = std::fs::read(&bin_path).unwrap();

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

    insta::assert_snapshot!(macho_container.data.len());
}

/// This test verify that a rpath can be added to a Mach-O file.
#[rstest]
fn test_rpath_add(#[files("tests/data/macho/fat/exec/*")] bin_path: PathBuf) {
    // load hello_with_rpath from data
    let data_bytes = std::fs::read(&bin_path).unwrap();

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

    insta::assert_snapshot!(macho_container.data.len());
}

/// This test verify that an dylib id can be changed in a Mach-O file.
#[rstest]
fn test_change_dylib_id(#[files("tests/data/macho/fat/libs/*")] lib_path: PathBuf) {
    let data_bytes = std::fs::read(&lib_path).unwrap();

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

    insta::assert_snapshot!(macho_container.data.len());
}

/// This test verify that a name of dynamic library can be changed
#[rstest]
fn test_change_dylib_name(#[files("tests/data/macho/fat/exec/*")] bin_path: PathBuf) {
    let data_bytes = std::fs::read(&bin_path).unwrap();

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

    insta::assert_snapshot!(macho_container.data.len());
}
