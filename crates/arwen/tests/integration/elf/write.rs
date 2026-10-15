//! Writing the modified file back.

use std::fs;
use std::process::Command;

use tempfile::tempdir;

use crate::common::ARWEN;

/// Linked with lld: no spare `.dynamic` entries and `.got` right after it
/// (`tools/elf/liblld.c`).
const LLD_LIB: &str = "../../tests/data/elf-lld/liblld.so";

/// Adding a dynamic entry to [`LLD_LIB`] means moving `.dynamic`, which the
/// writer refuses ("Overlapping immovable sections") only once it lays out the
/// new file. The file must be left as it was, not truncated.
#[test]
fn failed_modification_leaves_the_file_untouched() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("liblld.so");
    let original = fs::read(LLD_LIB).unwrap();
    fs::write(&file, &original).unwrap();

    let output = Command::new(ARWEN)
        .args(["elf", "add-needed"])
        .arg(&file)
        .arg("libbar.so")
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "add-needed unexpectedly succeeded"
    );
    assert_eq!(fs::read(&file).unwrap(), original);
}
