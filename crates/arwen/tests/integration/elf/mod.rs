//! Compare `arwen elf` against patchelf.
//!
//! Every modifying case applies the same operation with both tools to copies of
//! the same binary and compares what the dynamic loader would observe. Byte
//! equality is not expected: the two tools lay out the changes differently.
//!
//! patchelf is taken from `$ARWEN_PATCHELF`, falling back to `patchelf` on `PATH`.

use std::fs;
use std::path::{Path, PathBuf};

use goblin::elf::{dynamic, program_header, Elf};
use rstest::rstest;
use tempfile::tempdir;

use crate::common::ARWEN;

mod write;

/// Replaced by the path of the binary being modified.
const FILE: &str = "{file}";
/// Replaced by the RPATH/RUNPATH of the original binary.
const ORIG_RPATH: &str = "{rpath}";
/// Replaced by the path of a patchelf `--rename-dynamic-symbols` map file.
const RENAME_MAP: &str = "{map}";

const LONG_PATH: &str = "/a/very/long/path/that/does/not/fit/in/the/existing/section/\
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

/// Contents of `RENAME_MAP`, mirroring the `setlocale=my_setlocale` arwen argument.
const RENAME_MAP_CONTENTS: &str = "setlocale my_setlocale\n";

fn patchelf() -> String {
    std::env::var("ARWEN_PATCHELF").unwrap_or_else(|_| "patchelf".to_string())
}

/// Run `cmd args`, panicking with its output if it fails, and return stdout.
fn run(cmd: &str, args: &[String]) -> String {
    try_run(cmd, args).unwrap_or_else(|failure| panic!("{failure}"))
}

/// Run `cmd args` and return stdout, or a description of the failure.
fn try_run(cmd: &str, args: &[String]) -> Result<String, String> {
    let output = std::process::Command::new(cmd)
        .args(args)
        .output()
        .unwrap_or_else(|e| {
            panic!(
                "failed to spawn {cmd:?}: {e}\n\
                 The ELF integration tests need patchelf >= 0.18 on PATH, \
                 or its path in $ARWEN_PATCHELF."
            )
        });
    if !output.status.success() {
        return Err(format!(
            "{cmd} {args:?} failed with {}\nstdout: {}\nstderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        ));
    }
    Ok(String::from_utf8(output.stdout).unwrap())
}

/// Everything about an ELF file that the operations under test can change.
#[derive(Debug, PartialEq, Eq)]
struct ElfSummary {
    e_type: u16,
    os_abi: u8,
    interpreter: Option<String>,
    needed: Vec<String>,
    rpath: Vec<String>,
    runpath: Vec<String>,
    soname: Option<String>,
    flags_1: Option<u64>,
    has_debug: bool,
    gnu_stack_flags: Option<u32>,
    /// `(name, version)` of every dynamic symbol, sorted, since neither tool
    /// guarantees the symbol order is preserved.
    dynamic_symbols: Vec<(String, SymbolVersion)>,
}

impl ElfSummary {
    /// Describe every field in which `self` (arwen) differs from `other` (patchelf).
    fn differences(&self, other: &Self) -> Vec<String> {
        let mut differences = Vec::new();
        macro_rules! compare {
            ($($field:ident),*) => {$(
                if self.$field != other.$field {
                    differences.push(format!(
                        "{}: arwen={:?} patchelf={:?}",
                        stringify!($field),
                        self.$field,
                        other.$field
                    ));
                }
            )*};
        }
        compare!(
            e_type,
            os_abi,
            interpreter,
            needed,
            rpath,
            runpath,
            soname,
            flags_1,
            has_debug,
            gnu_stack_flags
        );
        for symbol in &self.dynamic_symbols {
            if !other.dynamic_symbols.contains(symbol) {
                differences.push(format!("dynamic symbol only in arwen: {symbol:?}"));
            }
        }
        for symbol in &other.dynamic_symbols {
            if !self.dynamic_symbols.contains(symbol) {
                differences.push(format!("dynamic symbol only in patchelf: {symbol:?}"));
            }
        }
        differences
    }
}

/// A symbol version resolved to names.
///
/// Version indices are not comparable between tools: arwen rebuilds the
/// version sections and may renumber them, patchelf keeps the original indices.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum SymbolVersion {
    /// No `.gnu.version` section.
    None,
    Local,
    Global,
    /// Version defined by this file (`.gnu.version_d`).
    Defined {
        name: String,
        hidden: bool,
    },
    /// Version required from a dependency (`.gnu.version_r`).
    Needed {
        file: String,
        name: String,
        hidden: bool,
    },
    /// Index that matches no version entry.
    Unknown(u16),
}

fn symbol_version(elf: &Elf, index: usize) -> SymbolVersion {
    use goblin::elf::symver::{VER_NDX_GLOBAL, VER_NDX_LOCAL};

    let Some(versym) = elf.versym.as_ref().and_then(|v| v.get_at(index)) else {
        return SymbolVersion::None;
    };
    let string = |offset| elf.dynstrtab.get_at(offset).unwrap_or_default().to_string();
    let hidden = versym.is_hidden();

    match versym.version() {
        VER_NDX_LOCAL => return SymbolVersion::Local,
        VER_NDX_GLOBAL => return SymbolVersion::Global,
        _ => {}
    }
    for need in elf.verneed.iter().flat_map(|section| section.iter()) {
        for aux in need.iter() {
            if aux.vna_other == versym.version() {
                return SymbolVersion::Needed {
                    file: string(need.vn_file),
                    name: string(aux.vna_name),
                    hidden,
                };
            }
        }
    }
    for def in elf.verdef.iter().flat_map(|section| section.iter()) {
        if def.vd_ndx == versym.version() {
            if let Some(aux) = def.iter().next() {
                return SymbolVersion::Defined {
                    name: string(aux.vda_name),
                    hidden,
                };
            }
        }
    }
    SymbolVersion::Unknown(versym.vs_val)
}

impl ElfSummary {
    fn read(path: &Path) -> Self {
        let data = fs::read(path).unwrap();
        let elf = Elf::parse(&data)
            .unwrap_or_else(|e| panic!("{} is not a valid ELF file: {e}", path.display()));

        let dyns = elf
            .dynamic
            .as_ref()
            .map(|d| d.dyns.as_slice())
            .unwrap_or_default();

        let mut dynamic_symbols: Vec<_> = elf
            .dynsyms
            .iter()
            .enumerate()
            .map(|(index, sym)| {
                let name = elf.dynstrtab.get_at(sym.st_name).unwrap_or_default();
                (name.to_string(), symbol_version(&elf, index))
            })
            .collect();
        dynamic_symbols.sort();

        Self {
            e_type: elf.header.e_type,
            os_abi: elf.header.e_ident[goblin::elf::header::EI_OSABI],
            interpreter: elf.interpreter.map(str::to_string),
            needed: elf.libraries.iter().map(|s| s.to_string()).collect(),
            rpath: elf.rpaths.iter().map(|s| s.to_string()).collect(),
            runpath: elf.runpaths.iter().map(|s| s.to_string()).collect(),
            soname: elf.soname.map(str::to_string),
            flags_1: dyns
                .iter()
                .find(|d| d.d_tag == dynamic::DT_FLAGS_1)
                .map(|d| d.d_val),
            has_debug: dyns.iter().any(|d| d.d_tag == dynamic::DT_DEBUG),
            gnu_stack_flags: elf
                .program_headers
                .iter()
                .find(|ph| ph.p_type == program_header::PT_GNU_STACK)
                .map(|ph| ph.p_flags),
            dynamic_symbols,
        }
    }
}

fn is_shared_object(path: &Path) -> bool {
    let data = fs::read(path).unwrap();
    Elf::parse(&data).unwrap().header.e_type == goblin::elf::header::ET_DYN
}

/// Copy `bin` into `dir` as `name` and expand the placeholders in `args` for it.
fn prepare(bin: &Path, dir: &Path, name: &str, args: &[&str]) -> (PathBuf, Vec<String>) {
    let file = dir.join(name);
    fs::copy(bin, &file).unwrap();

    let map = dir.join("rename.map");
    fs::write(&map, RENAME_MAP_CONTENTS).unwrap();

    let orig_rpath = run(
        &patchelf(),
        &["--print-rpath".into(), bin.display().to_string()],
    );
    let orig_rpath = orig_rpath.trim_end_matches('\n');

    let args = args
        .iter()
        .map(|arg| match *arg {
            FILE => file.display().to_string(),
            ORIG_RPATH => orig_rpath.to_string(),
            RENAME_MAP => map.display().to_string(),
            arg => arg.to_string(),
        })
        .collect();
    (file, args)
}

#[rstest]
#[case::set_interpreter(
    &["set-interpreter", "/lib64/test-ld.so", FILE],
    &["--set-interpreter", "/lib64/test-ld.so", FILE],
)]
#[case::set_interpreter_short(&["set-interpreter", "/ld.so", FILE], &["--set-interpreter", "/ld.so", FILE])]
#[case::set_interpreter_long(
    &["set-interpreter", LONG_PATH, FILE],
    &["--set-interpreter", LONG_PATH, FILE],
)]
#[case::set_rpath(
    &["set-rpath", "/opt/test/lib:/usr/local/lib", FILE],
    &["--set-rpath", "/opt/test/lib:/usr/local/lib", FILE],
)]
#[case::set_rpath_long(
    &["set-rpath", LONG_PATH, FILE],
    &["--set-rpath", LONG_PATH, FILE],
)]
#[case::add_rpath(
    &["add-rpath", "/opt/added/lib", FILE],
    &["--add-rpath", "/opt/added/lib", FILE],
)]
#[case::remove_rpath(&["remove-rpath", FILE], &["--remove-rpath", FILE])]
#[case::force_rpath(
    &["force-rpath", FILE],
    &["--force-rpath", "--set-rpath", ORIG_RPATH, FILE],
)]
#[case::add_needed(
    &["add-needed", FILE, "libfoo.so"],
    &["--add-needed", "libfoo.so", FILE],
)]
// patchelf inserts the new libraries in sorted order regardless of the
// command line; arwen keeps the given order.
// The names are given pre-sorted so both orders coincide.
#[case::add_needed_two(
    &["add-needed", FILE, "libbar.so", "libfoo.so"],
    &["--add-needed", "libbar.so", "--add-needed", "libfoo.so", FILE],
)]
#[case::remove_needed(
    &["remove-needed", FILE, "libdl.so.2"],
    &["--remove-needed", "libdl.so.2", FILE],
)]
#[case::replace_needed(
    &["replace-needed", FILE, "libc.so.6=libnotc.so.6"],
    &["--replace-needed", "libc.so.6", "libnotc.so.6", FILE],
)]
#[case::set_os_abi(
    &["set-os-abi", "freebsd", FILE],
    &["--set-os-abi", "freebsd", FILE],
)]
#[case::no_default_lib(&["no-default-lib", FILE], &["--no-default-lib", FILE])]
#[case::add_debug_tag(&["add-debug-tag", FILE], &["--add-debug-tag", FILE])]
#[case::clear_execstack(&["clear-exec-stack", FILE], &["--clear-execstack", FILE])]
#[case::set_execstack(&["set-exec-stack", FILE], &["--set-execstack", FILE])]
#[case::clear_symbol_version(
    &["clear-symbol-version", "setlocale", FILE],
    &["--clear-symbol-version", "setlocale", FILE],
)]
#[case::rename_dynamic_symbols(
    &["rename-dynamic-symbols", FILE, "setlocale=my_setlocale"],
    &["--rename-dynamic-symbols", RENAME_MAP, FILE],
)]
#[case::shrink_rpath(&["shrink-rpath", FILE], &["--shrink-rpath", FILE])]
#[case::set_soname(
    &["set-soname", "libnew.so", FILE],
    &["--set-soname", "libnew.so", FILE],
)]
fn modify_matches_patchelf(
    #[case] arwen_args: &[&str],
    #[case] patchelf_args: &[&str],
    #[files("../../tests/data/elf/*/exec/*")]
    #[files("../../tests/data/elf/*/lib/*")]
    bin: PathBuf,
    #[values(&[], &["--force-rpath", "--set-rpath", "/forced/lib"])]
    prepare_with_patchelf: &[&str],
) {
    let dir = tempdir().unwrap();

    // Modify variants of the input made by patchelf, e.g. with only a DT_RPATH.
    let bin = if prepare_with_patchelf.is_empty() {
        bin
    } else {
        let input = dir.path().join("input.bin");
        fs::copy(&bin, &input).unwrap();
        let args: Vec<String> = prepare_with_patchelf
            .iter()
            .map(|arg| arg.to_string())
            .chain([input.display().to_string()])
            .collect();
        run(&patchelf(), &args);
        input
    };

    let (arwen_file, arwen_args) = prepare(&bin, dir.path(), "arwen.bin", arwen_args);
    let (patchelf_file, patchelf_args) = prepare(&bin, dir.path(), "patchelf.bin", patchelf_args);

    let arwen_args: Vec<String> = ["elf".to_string()].into_iter().chain(arwen_args).collect();
    match (
        try_run(ARWEN, &arwen_args),
        try_run(&patchelf(), &patchelf_args),
    ) {
        (Ok(_), Ok(_)) => {}
        (Err(_), Err(_)) => {
            // Both refuse the operation (e.g. no `.interp` to change). arwen must
            // then leave the file exactly as it was.
            assert_eq!(
                fs::read(&arwen_file).unwrap(),
                fs::read(&bin).unwrap(),
                "arwen modified the file although the operation failed"
            );
            return;
        }
        // The one intended difference: patchelf silently leaves the soname of a
        // file that is not a shared object alone, arwen refuses to set it.
        (Err(_), Ok(_)) if arwen_args[1] == "set-soname" && !is_shared_object(&bin) => {
            assert_eq!(fs::read(&arwen_file).unwrap(), fs::read(&bin).unwrap());
            assert_eq!(fs::read(&patchelf_file).unwrap(), fs::read(&bin).unwrap());
            return;
        }
        (Err(failure), Ok(_)) => panic!("only arwen failed: {failure}"),
        (Ok(_), Err(failure)) => panic!("only patchelf failed: {failure}"),
    }

    // patchelf must still be able to parse what arwen wrote.
    run(
        &patchelf(),
        &["--print-needed".into(), arwen_file.display().to_string()],
    );

    let differences = ElfSummary::read(&arwen_file).differences(&ElfSummary::read(&patchelf_file));
    assert!(
        differences.is_empty(),
        "arwen and patchelf disagree:\n{}",
        differences.join("\n")
    );
}

#[rstest]
#[case::rpath("print-rpath", "--print-rpath")]
#[case::needed("print-needed", "--print-needed")]
#[case::interpreter("print-interpreter", "--print-interpreter")]
#[case::soname("print-soname", "--print-soname")]
#[case::execstack("print-exec-stack", "--print-execstack")]
#[case::os_abi("print-os-abi", "--print-os-abi")]
fn print_matches_patchelf(
    #[case] arwen_command: &str,
    #[case] patchelf_flag: &str,
    #[files("../../tests/data/elf/*/exec/*")]
    #[files("../../tests/data/elf/*/lib/*")]
    bin: PathBuf,
    #[values(&[], &["--remove-rpath"], &["--force-rpath", "--set-rpath", "/forced"], &["--set-execstack"])]
    prepare_with_patchelf: &[&str],
) {
    // Print from variants of the input made by patchelf, e.g. without any RPATH.
    let dir = tempdir().unwrap();
    let file = dir.path().join("input.bin");
    fs::copy(&bin, &file).unwrap();
    if !prepare_with_patchelf.is_empty() {
        let args: Vec<String> = prepare_with_patchelf
            .iter()
            .map(|arg| arg.to_string())
            .chain([file.display().to_string()])
            .collect();
        run(&patchelf(), &args);
    }

    let bin = file.display().to_string();
    let arwen = try_run(ARWEN, &["elf".into(), arwen_command.into(), bin.clone()]);
    let patchelf = try_run(&patchelf(), &[patchelf_flag.into(), bin]);
    match (arwen, patchelf) {
        (Ok(arwen), Ok(patchelf)) => assert_eq!(arwen, patchelf),
        // Both refuse, e.g. printing the interpreter of a shared library.
        (Err(_), Err(_)) => {}
        (Err(failure), Ok(_)) => panic!("only arwen failed: {failure}"),
        (Ok(_), Err(failure)) => panic!("only patchelf failed: {failure}"),
    }
}

/// Unlike patchelf, arwen does not add a `DT_NEEDED` entry that is already present.
#[rstest]
fn add_needed_existing_is_not_duplicated(
    #[files("../../tests/data/elf/*/exec/*")]
    #[files("../../tests/data/elf/*/lib/*")]
    bin: PathBuf,
) {
    let dir = tempdir().unwrap();
    let (file, args) = prepare(
        &bin,
        dir.path(),
        "arwen.bin",
        &["add-needed", FILE, "libc.so.6"],
    );

    let before = ElfSummary::read(&file);
    assert!(before.needed.iter().any(|lib| lib == "libc.so.6"));

    let args: Vec<String> = ["elf".to_string()].into_iter().chain(args).collect();
    run(ARWEN, &args);

    assert_eq!(ElfSummary::read(&file).needed, before.needed);
}
