use arwen_macho::MachoContainer;
use goblin::mach::MachO;
use rstest::rstest;
use std::path::PathBuf;

macro_rules! set_snapshot_suffix {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_suffix(format!($($expr,)*));
        let _guard = settings.bind_to_scope();
    }
}

/// This test checks if the rpath of a Mach-O file can be changed.
#[rstest]
fn test_rpath_change(#[files("../../tests/data/macho/x64/exec/*")] bin_path: PathBuf) {
    set_snapshot_suffix!("{}", bin_path.file_name().unwrap().to_str().unwrap());

    let data_bytes = std::fs::read(&bin_path).unwrap();

    let mut macho_container = MachoContainer::parse(&data_bytes).unwrap();

    macho_container
        .change_rpath("path_graf", "path_graf_path_graf_path_graf_path_graf")
        .unwrap();

    let changed_macho = MachO::parse(&macho_container.data, 0).unwrap();

    insta::assert_debug_snapshot!(changed_macho);
    insta::assert_debug_snapshot!(changed_macho.rpaths);
    insta::assert_snapshot!(macho_container.data.len());
}

/// This test checks if the rpath of a Mach-O file can be removed.
#[rstest]
fn test_rpath_remove(#[files("../../tests/data/macho/x64/exec/*")] bin_path: PathBuf) {
    set_snapshot_suffix!("{}", bin_path.file_name().unwrap().to_str().unwrap());
    let data_bytes = std::fs::read(&bin_path).unwrap();

    let mut macho_container = MachoContainer::parse(&data_bytes).unwrap();

    macho_container.remove_rpath("path_graf").unwrap();

    let changed_macho = MachO::parse(&macho_container.data, 0).unwrap();

    insta::assert_debug_snapshot!(changed_macho);
    insta::assert_debug_snapshot!(changed_macho.rpaths);
    insta::assert_snapshot!(macho_container.data.len());
}

/// This test verify that a rpath can be added to a Mach-O file.
#[rstest]
fn test_rpath_add(#[files("../../tests/data/macho/x64/exec/*")] bin_path: PathBuf) {
    set_snapshot_suffix!("{}", bin_path.file_name().unwrap().to_str().unwrap());
    let data_bytes = std::fs::read(&bin_path).unwrap();

    let mut macho_container = MachoContainer::parse(&data_bytes).unwrap();

    macho_container
        .add_rpath("abababababababababababaabbababababababababab")
        .unwrap();

    let changed_macho = MachO::parse(&macho_container.data, 0).unwrap();

    insta::assert_debug_snapshot!(changed_macho);
    insta::assert_debug_snapshot!(changed_macho.rpaths);
    insta::assert_snapshot!(macho_container.data.len());
}

/// This test verify that an dylib id can be changed in a Mach-O file.
#[rstest]
fn test_change_dylib_id(#[files("../../tests/data/macho/x64/libs/*")] lib_path: PathBuf) {
    let data_bytes = std::fs::read(&lib_path).unwrap();

    let mut macho_container = MachoContainer::parse(&data_bytes).unwrap();

    macho_container
        .change_install_id("very_very_very_very_very_very_very_longid.dylib")
        .unwrap();

    let changed_macho = MachO::parse(&macho_container.data, 0).unwrap();

    insta::assert_debug_snapshot!(changed_macho);
    insta::assert_debug_snapshot!(changed_macho.name);
    insta::assert_snapshot!(macho_container.data.len());
}

/// This test verify that a name of dynamic library can be changed
#[rstest]
fn test_change_dylib_name(#[files("../../tests/data/macho/x64/exec/*")] bin_path: PathBuf) {
    set_snapshot_suffix!("{}", bin_path.file_name().unwrap().to_str().unwrap());
    let data_bytes = std::fs::read(&bin_path).unwrap();

    let mut macho_container = MachoContainer::parse(&data_bytes).unwrap();

    macho_container
        .change_install_name(
            "/usr/lib/libSystem.B.dylib",
            "very_very_very_very_very_very_very_longid",
        )
        .unwrap();

    let changed_macho = MachO::parse(&macho_container.data, 0).unwrap();

    insta::assert_debug_snapshot!(changed_macho);
    insta::assert_debug_snapshot!(changed_macho.libs);
    insta::assert_snapshot!(macho_container.data.len());
}
