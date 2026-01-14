use arwen::elf::ElfContainer;
// use goblin::mach::MachO;
use goblin::elf::Elf;
use rstest::rstest;
use std::path::PathBuf;

/// This test checks if the rpath of a Elf file can be set.
#[rstest]
fn test_rpath_set(#[files("tests/data/elf/x64/exec/*")] bin_path: PathBuf) {
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
fn test_rpath_remove(#[files("tests/data/elf/x64/exec/*")] bin_path: PathBuf) {
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
fn test_rpath_add(#[files("tests/data/elf/x64/exec/*")] bin_path: PathBuf) {
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
fn test_force_rpath(#[files("tests/data/elf/x64/exec/*")] bin_path: PathBuf) {
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
fn test_set_interpreter(#[files("tests/data/elf/x64/exec/*")] bin_path: PathBuf) {
    // load hello_with_rpath from data
    let data_bytes = std::fs::read(&bin_path).unwrap();

    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();

    // we try to change the rpath of the file to something longer
    elf_container.set_interpreter("/my/ld-linux.so.2").unwrap();

    let mut changed_elf_data = Vec::new();
    elf_container.write(&mut changed_elf_data).unwrap();

    let changed_elf = Elf::parse(&changed_elf_data).unwrap();

    insta::assert_debug_snapshot!(changed_elf.interpreter);
    insta::assert_snapshot!(changed_elf_data.len());
}

/// This test verify that a rpath can be added to a Mach-O file.
#[rstest]
fn test_set_os_abi(#[files("tests/data/elf/x64/exec/*")] bin_path: PathBuf) {
    // load hello_with_rpath from data
    let data_bytes = std::fs::read(&bin_path).unwrap();

    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();

    // we try to change the rpath of the file to something longer
    elf_container.set_os_abi("freebsd").unwrap();

    let mut changed_elf_data = Vec::new();
    elf_container.write(&mut changed_elf_data).unwrap();

    let changed_elf = Elf::parse(&changed_elf_data).unwrap();

    insta::assert_debug_snapshot!(changed_elf.header);
    insta::assert_snapshot!(changed_elf_data.len());
}

/// This test verify that a rpath can be added to a Mach-O file.
#[rstest]
fn test_set_soname(#[files("tests/data/elf/x64/exec/*")] bin_path: PathBuf) {
    // load hello_with_rpath from data
    let data_bytes = std::fs::read(&bin_path).unwrap();

    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();

    // we try to change the rpath of the file to something longer
    elf_container.set_soname("my_soname").unwrap();

    let mut changed_elf_data = Vec::new();
    elf_container.write(&mut changed_elf_data).unwrap();

    let changed_elf = Elf::parse(&changed_elf_data).unwrap();

    insta::assert_debug_snapshot!(changed_elf.soname);
    insta::assert_snapshot!(changed_elf_data.len());
}

/// This test verify that a rpath can be added to a Mach-O file.
#[rstest]
fn test_add_dt_needed(#[files("tests/data/elf/x64/exec/*")] bin_path: PathBuf) {
    // load hello_with_rpath from data
    let data_bytes = std::fs::read(&bin_path).unwrap();

    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();

    // we try to change the rpath of the file to something longer
    elf_container
        .add_needed(vec!["my_dtneed".to_string()])
        .unwrap();

    let mut changed_elf_data = Vec::new();
    elf_container.write(&mut changed_elf_data).unwrap();

    let changed_elf = Elf::parse(&changed_elf_data).unwrap();

    insta::assert_debug_snapshot!(changed_elf.libraries);
    insta::assert_snapshot!(changed_elf_data.len());
}

/// This test verify that a rpath can be added to a Mach-O file.
#[rstest]
fn test_remove_dt_needed(#[files("tests/data/elf/x64/exec/*")] bin_path: PathBuf) {
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

    let changed_elf = Elf::parse(&changed_elf_data).unwrap();

    insta::assert_debug_snapshot!(changed_elf.libraries);
    insta::assert_snapshot!(changed_elf_data.len());
}

/// This test verify that a rpath can be added to a Mach-O file.
#[rstest]
fn test_no_default_lib(#[files("tests/data/elf/x64/exec/*")] bin_path: PathBuf) {
    // load hello_with_rpath from data
    let data_bytes = std::fs::read(&bin_path).unwrap();

    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();

    // we try to change the rpath of the file to something longer
    elf_container.no_default_lib().unwrap();

    let mut changed_elf_data = Vec::new();
    elf_container.write(&mut changed_elf_data).unwrap();

    let changed_elf = Elf::parse(&changed_elf_data).unwrap();

    insta::assert_debug_snapshot!(changed_elf.dynamic);
    insta::assert_snapshot!(changed_elf_data.len());
}

/// This test verify that a rpath can be added to a Mach-O file.
#[rstest]
fn test_add_debug_tag(#[files("tests/data/elf/x64/exec/*")] bin_path: PathBuf) {
    // load hello_with_rpath from data
    let data_bytes = std::fs::read(&bin_path).unwrap();

    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();

    // we try to change the rpath of the file to something longer
    elf_container.add_debug_tag().unwrap();

    let mut changed_elf_data = Vec::new();
    elf_container.write(&mut changed_elf_data).unwrap();

    let changed_elf = Elf::parse(&changed_elf_data).unwrap();

    insta::assert_debug_snapshot!(changed_elf.dynamic);
    insta::assert_snapshot!(changed_elf_data.len());
}

/// This test verify that a rpath can be added to a Mach-O file.
#[rstest]
fn test_set_execstack(#[files("tests/data/elf/x64/exec/*")] bin_path: PathBuf) {
    // load hello_with_rpath from data
    let data_bytes = std::fs::read(&bin_path).unwrap();

    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();

    // we try to change the rpath of the file to something longer
    elf_container.set_exec_stack().unwrap();

    let mut changed_elf_data = Vec::new();
    elf_container.write(&mut changed_elf_data).unwrap();

    let changed_elf = Elf::parse(&changed_elf_data).unwrap();

    insta::assert_debug_snapshot!(changed_elf.program_headers);
    insta::assert_snapshot!(changed_elf_data.len());
}

/// This test verify that a rpath can be added to a Mach-O file.
#[rstest]
fn test_clear_execstack(#[files("tests/data/elf/x64/exec/*")] bin_path: PathBuf) {
    // load hello_with_rpath from data
    let data_bytes = std::fs::read(&bin_path).unwrap();

    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();

    // we try to change the rpath of the file to something longer
    elf_container.set_exec_stack().unwrap();

    elf_container.clear_exec_stack().unwrap();

    let mut changed_elf_data = Vec::new();
    elf_container.write(&mut changed_elf_data).unwrap();

    let changed_elf = Elf::parse(&changed_elf_data).unwrap();

    insta::assert_debug_snapshot!(changed_elf.program_headers);
    insta::assert_snapshot!(changed_elf_data.len());
}

/// This test verifies that page size can be set for ELF files
#[rstest]
fn test_set_page_size(#[files("tests/data/elf/x64/exec/*")] bin_path: PathBuf) {
    let data_bytes = std::fs::read(&bin_path).unwrap();

    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();

    // Test getting default page size first
    let default_page_size = elf_container.get_page_size();
    assert!(default_page_size > 0);

    // Set a custom page size (8KB)
    elf_container.set_page_size(8192).unwrap();

    // Verify the page size was set correctly
    assert_eq!(elf_container.get_page_size(), 8192);

    let mut changed_elf_data = Vec::new();
    elf_container.write(&mut changed_elf_data).unwrap();

    let changed_elf = Elf::parse(&changed_elf_data).unwrap();

    // Verify PT_LOAD segments have correct alignment
    for ph in &changed_elf.program_headers {
        if ph.p_type == goblin::elf::program_header::PT_LOAD {
            assert_eq!(ph.p_align, 8192);
        }
    }

    insta::assert_debug_snapshot!(changed_elf.program_headers);
    insta::assert_snapshot!(changed_elf_data.len());
}

/// This test verifies that invalid page sizes are rejected
#[rstest]
fn test_set_page_size_validation(#[files("tests/data/elf/x64/exec/*")] bin_path: PathBuf) {
    let data_bytes = std::fs::read(&bin_path).unwrap();

    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();

    // Test invalid page sizes
    assert!(elf_container.set_page_size(1023).is_err()); // Too small
    assert!(elf_container.set_page_size(1025).is_err()); // Not power of 2
    assert!(elf_container.set_page_size(0).is_err()); // Zero

    // Test valid page sizes
    assert!(elf_container.set_page_size(1024).is_ok()); // Minimum valid
    assert!(elf_container.set_page_size(4096).is_ok()); // Standard
    assert!(elf_container.set_page_size(65536).is_ok()); // Large
}

/// This test verifies that shrink_rpath doesn't panic on invalid UTF-8 in DT_NEEDED.
#[rstest]
fn test_shrink_rpath_invalid_utf8_in_needed(
    #[files("tests/data/elf/x64/exec/*")] bin_path: PathBuf,
) {
    let data_bytes = std::fs::read(&bin_path).unwrap();
    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();

    // Set an absolute runpath so shrink_rpath will check directories
    elf_container.set_runpath("/tmp/nonexistent").unwrap();

    // DT_NEEDED entry with invalid UTF-8 bytes
    let invalid_utf8: Vec<u8> = vec![b'l', b'i', b'b', 0xFF, 0xFE, b'.', b's', b'o'];
    elf_container.inner.elf_add_needed(&[invalid_utf8]).unwrap();

    elf_container.shrink_rpath(vec![]).unwrap();
}
