use arwen::patcher::change_rpath;
use goblin::mach::MachO;
use std::path::PathBuf;

/// This module tests the changing of `rpath` in macho files.
///

#[test]
fn test_rpath_change() {
    // load hello_with_rpath from data
    let package_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data = package_dir.join("tests/data/hello_with_rpath");

    let data_bytes = std::fs::read(&data).unwrap();

    let changed = change_rpath(data_bytes);

    let changed_macho = MachO::parse(&changed, 0).unwrap();

    // find changed rpath command
    let changed_rpath = changed_macho
        .load_commands
        .iter()
        .find(|load_command| {
            if let goblin::mach::load_command::CommandVariant::Rpath(rpath_command) =
                &load_command.command
            {
                return true;
            } else {
                return false;
            };
        })
        .unwrap();

    insta::assert_debug_snapshot!(changed_rpath);
    insta::assert_debug_snapshot!(changed_macho.rpaths);
}
