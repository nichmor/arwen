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

/// `PT_INTERP` must cover exactly the new interpreter and its NUL terminator,
/// whether or not the `.interp` section had to move. The kernel rejects the
/// file if the last byte of the segment is not NUL.
#[rstest]
fn test_set_interpreter_updates_pt_interp(
    #[files("../../tests/data/elf/*/exec/*")] bin_path: PathBuf,
    #[values(
        "/ld.so",
        "/lib64/test-ld.so",
        "/a/much/longer/interpreter/path/that/needs/more/room/ld.so"
    )]
    interpreter: &str,
) {
    let data_bytes = std::fs::read(&bin_path).unwrap();
    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();
    elf_container.set_interpreter(interpreter).unwrap();

    let mut changed_elf_data = Vec::new();
    elf_container.write(&mut changed_elf_data).unwrap();

    let changed_elf = Elf::parse(&changed_elf_data).unwrap();
    assert_eq!(changed_elf.interpreter, Some(interpreter));

    let interp = changed_elf
        .program_headers
        .iter()
        .find(|ph| ph.p_type == goblin::elf::program_header::PT_INTERP)
        .unwrap();
    assert_eq!(interp.p_filesz, interpreter.len() as u64 + 1);
    let end = (interp.p_offset + interp.p_filesz) as usize;
    assert_eq!(changed_elf_data[end - 1], 0);
}

/// Modifications must not shrink segments that linkers pad on purpose.
///
/// `PT_GNU_RELRO` ends on a page boundary so that the dynamic loader, which
/// rounds the end down, also protects the last page. Only `PT_INTERP` and
/// `PT_DYNAMIC` track their section exactly.
#[rstest]
fn test_padded_segments_are_not_shrunk(
    #[files("../../tests/data/elf/*/exec/*")]
    #[files("../../tests/data/elf/*/lib/*")]
    bin_path: PathBuf,
    #[values("set_rpath", "remove_rpath", "add_needed", "remove_needed")] operation: &str,
) {
    use goblin::elf::program_header::{PT_DYNAMIC, PT_INTERP, PT_LOAD};

    let data_bytes = std::fs::read(&bin_path).unwrap();
    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();
    match operation {
        "set_rpath" => elf_container.set_runpath("/x").unwrap(),
        "remove_rpath" => elf_container.remove_runpath().unwrap(),
        "add_needed" => elf_container.add_needed(vec!["libextra.so"]).unwrap(),
        "remove_needed" => elf_container.remove_needed(vec!["libc.so.6"]).unwrap(),
        _ => unreachable!(),
    }
    let mut changed_elf_data = Vec::new();
    elf_container.write(&mut changed_elf_data).unwrap();

    let original = Elf::parse(&data_bytes).unwrap();
    let changed = Elf::parse(&changed_elf_data).unwrap();
    for old in &original.program_headers {
        if matches!(old.p_type, PT_LOAD | PT_INTERP | PT_DYNAMIC) {
            continue;
        }
        let Some(new) = changed.program_headers.iter().find(|ph| {
            ph.p_type == old.p_type
                && ph.p_vaddr <= old.p_vaddr
                && old.p_vaddr < ph.p_vaddr + ph.p_memsz.max(1)
        }) else {
            // Segments without size (e.g. PT_GNU_STACK) are matched by type alone.
            assert!(
                changed
                    .program_headers
                    .iter()
                    .any(|ph| ph.p_type == old.p_type),
                "segment {old:?} disappeared"
            );
            continue;
        };
        assert!(
            new.p_vaddr + new.p_memsz >= old.p_vaddr + old.p_memsz,
            "segment shrank:\nold: {old:?}\nnew: {new:?}"
        );
        assert!(
            new.p_offset + new.p_filesz >= old.p_offset + old.p_filesz,
            "segment shrank:\nold: {old:?}\nnew: {new:?}"
        );
    }
}

/// `replace-needed` must also rename the library in the version requirements
/// (`.gnu.version_r`): the dynamic loader looks the versions up in the loaded
/// object of that name.
#[rstest]
fn test_replace_needed_renames_version_requirements(
    #[files("../../tests/data/elf/*/*/*")] bin_path: PathBuf,
) {
    let data_bytes = std::fs::read(&bin_path).unwrap();
    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();
    elf_container
        .replace_needed(&std::collections::HashMap::from([(
            "libc.so.6",
            "libnotc.so.6",
        )]))
        .unwrap();
    let mut changed_elf_data = Vec::new();
    elf_container.write(&mut changed_elf_data).unwrap();

    let version_files = |data: &[u8]| -> Vec<String> {
        let elf = Elf::parse(data).unwrap();
        let verneed = elf
            .verneed
            .as_ref()
            .expect("fixture has version requirements");
        verneed
            .iter()
            .map(|need| elf.dynstrtab.get_at(need.vn_file).unwrap().to_string())
            .collect()
    };
    let expected: Vec<String> = version_files(&data_bytes)
        .into_iter()
        .map(|file| {
            if file == "libc.so.6" {
                "libnotc.so.6".to_string()
            } else {
                file
            }
        })
        .collect();
    assert!(expected.contains(&"libnotc.so.6".to_string()));
    assert_eq!(version_files(&changed_elf_data), expected);
}

/// Only shared objects get a soname: setting one on an executable (`ET_EXEC`)
/// fails and leaves it without one.
#[rstest]
fn test_set_soname_requires_a_shared_object(
    #[files("../../tests/data/elf/*/*/*")] bin_path: PathBuf,
) {
    let data_bytes = std::fs::read(&bin_path).unwrap();
    let is_shared = Elf::parse(&data_bytes).unwrap().header.e_type == goblin::elf::header::ET_DYN;

    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();
    let result = elf_container.set_soname("libnew.so.1");
    let mut out = Vec::new();
    elf_container.write(&mut out).unwrap();
    if is_shared {
        result.unwrap();
        assert_eq!(Elf::parse(&out).unwrap().soname, Some("libnew.so.1"));
    } else {
        assert!(result.is_err());
        assert_eq!(Elf::parse(&out).unwrap().soname, None);
    }
}

/// Changing the search path of a file with only a `DT_RPATH` turns it into a
/// `DT_RUNPATH`.
#[rstest]
fn test_lone_rpath_becomes_runpath(
    #[files("../../tests/data/elf/*/*/*")] bin_path: PathBuf,
    #[values("set", "add", "shrink")] operation: &str,
) {
    let data_bytes = std::fs::read(&bin_path).unwrap();
    // Start from a lone DT_RPATH.
    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();
    elf_container.set_runpath("/opt/old/lib:$ORIGIN").unwrap();
    elf_container.force_rpath().unwrap();
    let mut rpath_only = Vec::new();
    elf_container.write(&mut rpath_only).unwrap();
    let before = Elf::parse(&rpath_only).unwrap();
    assert!(before.runpaths.is_empty());
    assert_eq!(before.rpaths, vec!["/opt/old/lib:$ORIGIN"]);

    let mut elf_container = ElfContainer::parse(&rpath_only).unwrap();
    let expected = match operation {
        "set" => {
            elf_container.set_runpath("/new/lib").unwrap();
            "/new/lib".to_string()
        }
        "add" => {
            elf_container.add_runpath("/new/lib").unwrap();
            "/opt/old/lib:$ORIGIN:/new/lib".to_string()
        }
        "shrink" => {
            // /opt/old/lib does not exist; relative entries are kept.
            elf_container.shrink_rpath(Vec::new()).unwrap();
            "$ORIGIN".to_string()
        }
        _ => unreachable!(),
    };
    let mut changed_elf_data = Vec::new();
    elf_container.write(&mut changed_elf_data).unwrap();

    let changed = Elf::parse(&changed_elf_data).unwrap();
    assert!(changed.rpaths.is_empty(), "{:?}", changed.rpaths);
    assert_eq!(changed.runpaths, vec![expected.as_str()]);
}

/// Linked with lld: no `DT_NEEDED`, no search path and no symbol versions.
const LLD_LIB: &str = "../../tests/data/elf-lld/liblld.so";

/// `clear-symbol-version` fails on a file without symbol versions.
#[test]
fn test_clear_symbol_version_requires_symbol_versions() {
    let data_bytes = std::fs::read(LLD_LIB).unwrap();
    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();
    assert!(matches!(
        elf_container.clear_version_symbol("bar_next"),
        Err(arwen_elf::ElfError::MissingSection(".gnu.version"))
    ));
}

/// `shrink-rpath` leaves a file without a search path alone.
#[test]
fn test_shrink_rpath_without_search_path() {
    let data_bytes = std::fs::read(LLD_LIB).unwrap();
    let mut elf_container = ElfContainer::parse(&data_bytes).unwrap();
    elf_container.shrink_rpath(Vec::new()).unwrap();
    let mut changed_elf_data = Vec::new();
    elf_container.write(&mut changed_elf_data).unwrap();

    let changed = Elf::parse(&changed_elf_data).unwrap();
    assert!(changed.rpaths.is_empty());
    assert!(changed.runpaths.is_empty());
}
