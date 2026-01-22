use arwen_elf::ElfContainer;
use goblin::elf::Elf;
use rstest::rstest;
use std::path::PathBuf;

/// This test checks if the rpath of a Elf file can be set.
#[rstest]
fn test_rpath_set(#[files("../../tests/data/elf/x64/exec/*")] bin_path: PathBuf) {
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

/// This test checks if the rpath of an ELF file can be removed.
#[rstest]
fn test_rpath_remove(#[files("../../tests/data/elf/x64/exec/*")] bin_path: PathBuf) {
    let data_bytes = std::fs::read(&bin_path).unwrap();

    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();

    elf_container.remove_runpath().unwrap();

    let mut changed_elf_data = Vec::new();
    elf_container.write(&mut changed_elf_data).unwrap();

    let changed_elf = Elf::parse(&changed_elf_data).unwrap();

    insta::assert_debug_snapshot!(changed_elf.runpaths);
    insta::assert_snapshot!(changed_elf_data.len());
}

/// This test verify that a rpath can be added to an ELF file.
#[rstest]
fn test_rpath_add(#[files("../../tests/data/elf/x64/exec/*")] bin_path: PathBuf) {
    let data_bytes = std::fs::read(&bin_path).unwrap();

    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();

    elf_container
        .add_runpath("abababababababababababaabbababababababababab")
        .unwrap();

    let mut changed_elf_data = Vec::new();
    elf_container.write(&mut changed_elf_data).unwrap();

    let changed_macho = Elf::parse(&changed_elf_data).unwrap();

    insta::assert_debug_snapshot!(changed_macho.runpaths);
    insta::assert_snapshot!(changed_elf_data.len());
}

/// This test verify that force_rpath works.
#[rstest]
fn test_force_rpath(#[files("../../tests/data/elf/x64/exec/*")] bin_path: PathBuf) {
    let data_bytes = std::fs::read(&bin_path).unwrap();

    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();

    elf_container.force_rpath().unwrap();

    let mut changed_elf_data = Vec::new();
    elf_container.write(&mut changed_elf_data).unwrap();

    let changed_macho = Elf::parse(&changed_elf_data).unwrap();

    insta::assert_debug_snapshot!(changed_macho.rpaths);
    insta::assert_snapshot!(changed_elf_data.len());
}

/// This test verifies that page size can be set for ELF files
#[rstest]
fn test_set_page_size(#[files("../../tests/data/elf/x64/exec/*")] bin_path: PathBuf) {
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
