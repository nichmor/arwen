use arwen::elf::ElfContainer;
// use goblin::mach::MachO;
use goblin::elf::Elf;
use rstest::rstest;
use std::path::PathBuf;

/// This test checks if the rpath of a Elf file can be set.
#[rstest]
fn test_rpath_set(#[files("tests/data/elf/x32/exec/*")] bin_path: PathBuf) {
    let data_bytes = std::fs::read(&bin_path).unwrap();

    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();

    elf_container
        .set_runpath("path_graf_path_graf_path_graf_path_graf")
        .unwrap();

    let mut changed_elf_data = Vec::new();
    elf_container.write(&mut changed_elf_data).unwrap();

    let changed_elf = Elf::parse(&changed_elf_data).unwrap();

    insta::assert_debug_snapshot!(changed_elf.runpaths);

    insta::assert_snapshot!(changed_elf_data.len());
}

/// This test checks if the rpath of a Mach-O file can be removed.
#[rstest]
fn test_rpath_remove(#[files("tests/data/elf/x32/exec/*")] bin_path: PathBuf) {
    let data_bytes = std::fs::read(&bin_path).unwrap();

    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();

    // we try to change the rpath of the file to something longer
    elf_container.remove_runpath().unwrap();

    let mut changed_elf_data = Vec::new();
    elf_container.write(&mut changed_elf_data).unwrap();

    let changed_elf = Elf::parse(&changed_elf_data).unwrap();

    insta::assert_debug_snapshot!(changed_elf.runpaths);
    insta::assert_snapshot!(changed_elf_data.len());
}

/// This test verify that a rpath can be added to a Mach-O file.
#[rstest]
fn test_rpath_add(#[files("tests/data/elf/x32/exec/*")] bin_path: PathBuf) {
    // load hello_with_rpath from data
    let data_bytes = std::fs::read(&bin_path).unwrap();

    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();

    // we try to change the rpath of the file to something longer
    elf_container
        .add_runpath("abababababababababababaabbababababababababab")
        .unwrap();

    let mut changed_elf_data = Vec::new();
    elf_container.write(&mut changed_elf_data).unwrap();

    let changed_macho = Elf::parse(&changed_elf_data).unwrap();

    insta::assert_debug_snapshot!(changed_macho.runpaths);
    insta::assert_snapshot!(changed_elf_data.len());
}

/// This test verify that a rpath can be added to a Mach-O file.
#[rstest]
fn test_force_rpath(#[files("tests/data/elf/x32/exec/*")] bin_path: PathBuf) {
    // load hello_with_rpath from data
    let data_bytes = std::fs::read(&bin_path).unwrap();

    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();

    // we try to change the rpath of the file to something longer
    elf_container.force_rpath().unwrap();

    let mut changed_elf_data = Vec::new();
    elf_container.write(&mut changed_elf_data).unwrap();

    let changed_macho = Elf::parse(&changed_elf_data).unwrap();

    insta::assert_debug_snapshot!(changed_macho.rpaths);
    insta::assert_snapshot!(changed_elf_data.len());
}

/// This test verify that a rpath can be added to a Mach-O file.
#[rstest]
fn test_set_interpreter(#[files("tests/data/elf/x32/exec/*")] bin_path: PathBuf) {
    // load hello_with_rpath from data
    let data_bytes = std::fs::read(&bin_path).unwrap();

    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();

    // we try to change the rpath of the file to something longer
    elf_container.set_interpreter("/my/ld-linux.so.2").unwrap();

    let mut changed_elf_data = Vec::new();
    elf_container.write(&mut changed_elf_data).unwrap();

    let changed_macho = Elf::parse(&changed_elf_data).unwrap();

    insta::assert_debug_snapshot!(changed_macho.interpreter);
    insta::assert_snapshot!(changed_elf_data.len());
}

/// This test verify that a rpath can be added to a Mach-O file.
#[rstest]
fn test_set_os_abi(#[files("tests/data/elf/x32/exec/*")] bin_path: PathBuf) {
    // load hello_with_rpath from data
    let data_bytes = std::fs::read(&bin_path).unwrap();

    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();

    // we try to change the rpath of the file to something longer
    elf_container.set_os_abi("freebsd").unwrap();

    let mut changed_elf_data = Vec::new();
    elf_container.write(&mut changed_elf_data).unwrap();

    let changed_macho = Elf::parse(&changed_elf_data).unwrap();

    insta::assert_debug_snapshot!(changed_macho.header);
    insta::assert_snapshot!(changed_elf_data.len());
}

/// This test verify that a rpath can be added to a Mach-O file.
#[rstest]
fn test_set_soname(#[files("tests/data/elf/x32/exec/*")] bin_path: PathBuf) {
    // load hello_with_rpath from data
    let data_bytes = std::fs::read(&bin_path).unwrap();

    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();

    // we try to change the rpath of the file to something longer
    elf_container.set_soname("my_soname").unwrap();

    let mut changed_elf_data = Vec::new();
    elf_container.write(&mut changed_elf_data).unwrap();

    let changed_macho = Elf::parse(&changed_elf_data).unwrap();

    insta::assert_debug_snapshot!(changed_macho.soname);
    insta::assert_snapshot!(changed_elf_data.len());
}

/// This test verify that a rpath can be added to a Mach-O file.
#[rstest]
fn test_add_dt_needed(#[files("tests/data/elf/x32/exec/*")] bin_path: PathBuf) {
    // load hello_with_rpath from data
    let data_bytes = std::fs::read(&bin_path).unwrap();

    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();

    // we try to change the rpath of the file to something longer
    elf_container
        .add_needed(vec!["my_dtneed".to_string()])
        .unwrap();

    let mut changed_elf_data = Vec::new();
    elf_container.write(&mut changed_elf_data).unwrap();

    let changed_macho = Elf::parse(&changed_elf_data).unwrap();

    insta::assert_debug_snapshot!(changed_macho.libraries);
    insta::assert_snapshot!(changed_elf_data.len());
}

/// This test verify that a rpath can be added to a Mach-O file.
#[rstest]
fn test_remove_dt_needed(#[files("tests/data/elf/x32/exec/*")] bin_path: PathBuf) {
    // load hello_with_rpath from data
    let data_bytes = std::fs::read(&bin_path).unwrap();

    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();

    // we try to change the rpath of the file to something longer
    elf_container
        .add_needed(vec!["my_dtneed".to_string()])
        .unwrap();

    // we try to change the rpath of the file to something longer
    elf_container
        .add_needed(vec!["my_dtneed_to_remove".to_string()])
        .unwrap();

    // we try to change the rpath of the file to something longer
    elf_container
        .remove_needed(vec!["my_dtneed_to_remove".to_string()])
        .unwrap();

    let mut changed_elf_data = Vec::new();
    elf_container.write(&mut changed_elf_data).unwrap();

    let changed_macho = Elf::parse(&changed_elf_data).unwrap();

    insta::assert_debug_snapshot!(changed_macho.libraries);
    insta::assert_snapshot!(changed_elf_data.len());
}

/// This test verify that a rpath can be added to a Mach-O file.
#[rstest]
fn test_no_default_lib(#[files("tests/data/elf/x32/exec/*")] bin_path: PathBuf) {
    // load hello_with_rpath from data
    let data_bytes = std::fs::read(&bin_path).unwrap();

    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();

    // we try to change the rpath of the file to something longer
    elf_container.no_default_lib().unwrap();

    let mut changed_elf_data = Vec::new();
    elf_container.write(&mut changed_elf_data).unwrap();

    let changed_macho = Elf::parse(&changed_elf_data).unwrap();

    insta::assert_debug_snapshot!(changed_macho.dynamic);
    insta::assert_snapshot!(changed_elf_data.len());
}

/// This test verify that a rpath can be added to a Mach-O file.
#[rstest]
fn test_add_debug_tag(#[files("tests/data/elf/x32/exec/*")] bin_path: PathBuf) {
    // load hello_with_rpath from data
    let data_bytes = std::fs::read(&bin_path).unwrap();

    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();

    // we try to change the rpath of the file to something longer
    elf_container.add_debug_tag().unwrap();

    let mut changed_elf_data = Vec::new();
    elf_container.write(&mut changed_elf_data).unwrap();

    let changed_macho = Elf::parse(&changed_elf_data).unwrap();

    insta::assert_debug_snapshot!(changed_macho.dynamic);
    insta::assert_snapshot!(changed_elf_data.len());
}

/// This test verify that a rpath can be added to a Mach-O file.
#[rstest]
fn test_set_execstack(#[files("tests/data/elf/x32/exec/*")] bin_path: PathBuf) {
    // load hello_with_rpath from data
    let data_bytes = std::fs::read(&bin_path).unwrap();

    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();

    // we try to change the rpath of the file to something longer
    elf_container.set_exec_stack().unwrap();

    let mut changed_elf_data = Vec::new();
    elf_container.write(&mut changed_elf_data).unwrap();

    let changed_macho = Elf::parse(&changed_elf_data).unwrap();

    insta::assert_debug_snapshot!(changed_macho.program_headers);
    insta::assert_snapshot!(changed_elf_data.len());
}

/// This test verify that a rpath can be added to a Mach-O file.
#[rstest]
fn test_clear_execstack(#[files("tests/data/elf/x32/exec/*")] bin_path: PathBuf) {
    // load hello_with_rpath from data
    let data_bytes = std::fs::read(&bin_path).unwrap();

    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();

    // we try to change the rpath of the file to something longer
    elf_container.set_exec_stack().unwrap();

    elf_container.clear_exec_stack().unwrap();

    let mut changed_elf_data = Vec::new();
    elf_container.write(&mut changed_elf_data).unwrap();

    let changed_macho = Elf::parse(&changed_elf_data).unwrap();

    insta::assert_debug_snapshot!(changed_macho.program_headers);
    insta::assert_snapshot!(changed_elf_data.len());
}
