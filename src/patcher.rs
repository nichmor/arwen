use std::vec;

use goblin::mach::load_command::{CommandVariant::*, DylibCommand};
use goblin::mach::{
    header::Header64,
    load_command::{self, LoadCommand, RpathCommand},
    MachO,
};


use scroll::Pwrite;

use crate::commands::{DlibCommandBuilder, RpathCommandBuilder};

/// Change the rpath of a Mach-O file.
pub fn change_rpath(bytes_of_file: Vec<u8>, old_rpath: String, new_rpath: String) -> Vec<u8> {
    let parsed_macho = MachO::parse(&bytes_of_file, 0).unwrap();

    // TODO: goblin doesn't export parsed data
    // so we mimic it by using directly the bytes of the file
    // let mut buffer = parsed_macho.data.to_vec();
    let mut buffer = bytes_of_file.to_vec();

    let header = parsed_macho.header;

    let mut header: Header64 = header.into();

    // let's start with load commands
    // initial_offset += wroted;
    let old_rpath_index = parsed_macho
        .rpaths
        .iter()
        .position(|rpath| rpath == &old_rpath)
        .expect("rpath should exist");

    // now based on the index, we need to find the RpathCommand from the load commands
    let (load_command, _rpath_command) =
        find_rpath_command(&parsed_macho.load_commands, old_rpath_index)
            .expect("rpath command should exist");

    remove_load_command(&mut buffer, &mut header, load_command);

    let (new_rpath, new_rpath_command_buffer) =
        RpathCommandBuilder::new(new_rpath.as_str()).build();

    insert_command(
        &mut buffer,
        &mut header,
        load_command.offset,
        new_rpath.cmdsize,
        new_rpath_command_buffer,
    );

    buffer.to_vec()
}

/// Add a new rpath to a Mach-O file.
pub fn add_rpath(bytes_of_file: Vec<u8>, new_rpath: String) -> Vec<u8> {
    // let's calculate the total size of all the header and commands
    let parsed_macho = MachO::parse(&bytes_of_file, 0).unwrap();

    // TODO: goblin doesn't export parsed data
    // so we mimic it by using directly the bytes of the file
    // let mut buffer = parsed_macho.data.to_vec();
    let mut buffer = bytes_of_file.to_vec();

    let header = parsed_macho.header;

    let mut header: Header64 = header.into();

    let (new_rpath, new_rpath_command_buffer) =
        RpathCommandBuilder::new(new_rpath.as_str()).build();

    let offset_size = header.size() + header.sizeofcmds as usize;

    insert_command(
        &mut buffer,
        &mut header,
        offset_size,
        new_rpath.cmdsize,
        new_rpath_command_buffer,
    );

    buffer.to_vec()
}

/// Remove a specified rpath from a Mach-O file.
pub fn remove_rpath(bytes_of_file: Vec<u8>, old_rpath: String) -> Vec<u8> {
    let parsed_macho = MachO::parse(&bytes_of_file, 0).unwrap();

    // TODO: goblin doesn't export parsed data
    // so we mimic it by using directly the bytes of the file
    // let mut buffer = parsed_macho.data.to_vec();
    let mut buffer = bytes_of_file.to_vec();

    let header = parsed_macho.header;

    let mut header: Header64 = header.into();

    let old_rpath_index = parsed_macho
        .rpaths
        .iter()
        .position(|rpath| rpath == &old_rpath)
        .expect("rpath should exist");

    // now based on the index, we need to find the RpathCommand from the load commands
    let (load_command, _rpath_command) =
        find_rpath_command(&parsed_macho.load_commands, old_rpath_index)
            .expect("rpath command should exist");

    remove_load_command(&mut buffer, &mut header, load_command);

    buffer.to_vec()
}

/// Change the install name of a dylib in a Mach-O file.
pub fn change_install_name(bytes_of_file: Vec<u8>, old_name: String, new_name: String) -> Vec<u8> {
    let parsed_macho = MachO::parse(&bytes_of_file, 0).unwrap();

    // TODO: goblin doesn't export parsed data
    // so we mimic it by using directly the bytes of the file
    // let mut buffer = parsed_macho.data.to_vec();
    let mut buffer = bytes_of_file.to_vec();

    let header = parsed_macho.header;

    let mut header: Header64 = header.into();

    // let's start with load commands
    let old_dylib = parsed_macho
        .libs
        .iter()
        .position(|name| name == &old_name)
        .expect("dylib name should exist");

    // now based on the index, we need to find the RpathCommand from the load commands
    // we use -1 because dylib contains self, so we need to omit it
    let (load_command, old_dylib) = find_dylib_command(&parsed_macho.load_commands, old_dylib - 1)
        .expect("rpath command should exist");

    remove_load_command(&mut buffer, &mut header, load_command);

    let (new_dylib, new_dylib_command_buffer) =
        DlibCommandBuilder::new(&new_name, *old_dylib).build();

    insert_command(
        &mut buffer,
        &mut header,
        load_command.offset,
        new_dylib.cmdsize,
        new_dylib_command_buffer,
    );

    buffer.to_vec()
}

/// Change the install id of a dylib in a Mach-O file.
pub fn change_install_id(bytes_of_file: Vec<u8>, new_id: String) -> Vec<u8> {
    let parsed_macho = MachO::parse(&bytes_of_file, 0).unwrap();

    // TODO: goblin doesn't export parsed data
    // so we mimic it by using directly the bytes of the file
    // let mut buffer = parsed_macho.data.to_vec();
    let mut buffer = bytes_of_file.to_vec();

    let header = parsed_macho.header;

    let mut header: Header64 = header.into();

    // now based on the index, we need to find the RpathCommand from the load commands
    let (load_command, old_dylib) = find_dylib_id(&parsed_macho.load_commands)
        .expect("LC_ID_DYLIB is missing or file is not a shared library");

    remove_load_command(&mut buffer, &mut header, load_command);

    let (new_dylib, new_dylib_command_buffer) =
        DlibCommandBuilder::new(&new_id, *old_dylib).build();

    insert_command(
        &mut buffer,
        &mut header,
        load_command.offset,
        new_dylib.cmdsize,
        new_dylib_command_buffer,
    );

    buffer.to_vec()
}

/// Removes a load command from the buffer.
///
/// # Arguments
/// * `buffer` - Mutable byte buffer representing the Mach-O file.
/// * `header` - Header of the macho. It will be updated after removing the load command.
/// * `load_command` - Load Command to remove.
pub fn remove_load_command(
    buffer: &mut Vec<u8>,
    header: &mut Header64,
    load_command: &LoadCommand,
) {
    // // remove entire command from the buffer
    eprintln!("buffer before {}", buffer.len());

    let drain_offset = load_command.offset + load_command.command.cmdsize();
    buffer.drain(load_command.offset..drain_offset);

    eprintln!("buffer after {}", buffer.len());

    // update the header
    header.ncmds -= 1;
    header.sizeofcmds -= load_command.command.cmdsize() as u32;

    // Step 3: Insert padding after the remaining load commands
    let padding_offset = header.size() + header.sizeofcmds as usize;
    let padding_size = load_command.command.cmdsize();

    // Ensure there's enough space for the padding
    if padding_offset + padding_size > buffer.len() {
        buffer.resize(padding_offset + padding_size, 0);
    }

    // Write zero bytes as padding
    let mut zeroing_buffer = vec![0u8; padding_size];
    zeroing_buffer.fill(0);
    eprintln!("zeroing buffer {}", zeroing_buffer.len());

    let tail = buffer.split_off(padding_offset);

    // Extend with the new slice
    buffer.extend(&zeroing_buffer);

    // Add back the tail
    buffer.extend(tail);

    buffer.pwrite(*header, 0).unwrap();
}

/// Insert a new load command at the given offset.
///
/// # Arguments
/// * `buffer` - Mutable byte buffer representing the Mach-O file.
/// * `header` - Header of the macho. It will be updated after removing the load command.
/// * `load_command` - Load Command to remove.
pub fn insert_command(
    buffer: &mut Vec<u8>,
    header: &mut Header64,
    offset: usize,
    new_cmd_size: u32,
    load_data: Vec<u8>,
) {
    // update the header
    header.ncmds += 1;
    header.sizeofcmds += new_cmd_size;

    // write new command
    let tail = buffer.split_off(offset);

    // Extend with the new slice
    buffer.extend(&load_data);

    // Add back the tail
    buffer.extend(tail);

    let drain_offset = header.size() + header.sizeofcmds as usize + new_cmd_size as usize;
    buffer.drain(header.size() + header.sizeofcmds as usize..drain_offset);

    buffer.pwrite(*header, 0).unwrap();
}

/// Find the rpath command at the given index.
pub fn find_rpath_command(
    commands: &[load_command::LoadCommand],
    index: usize,
) -> Option<(&LoadCommand, &RpathCommand)> {
    let mut count = 0;

    for command in commands {
        if let Rpath(rpath_command) = &command.command {
            if count == index {
                return Some((command, rpath_command));
            }
            count += 1;
        }
    }

    None
}

/// Find the dylib command at the given index.
pub fn find_dylib_command(
    commands: &[load_command::LoadCommand],
    index: usize,
) -> Option<(&LoadCommand, &DylibCommand)> {
    let mut count = 0;

    for command in commands {
        eprintln!("command {:?}", command);
        match &command.command {
            LoadDylib(dylib_command)
            | LoadUpwardDylib(dylib_command)
            | ReexportDylib(dylib_command)
            | LoadWeakDylib(dylib_command)
            | LazyLoadDylib(dylib_command) => {
                if count == index {
                    return Some((command, dylib_command));
                }
                count += 1;
            }
            _ => {}
        }
    }

    None
}

/// Find the dylib id command.
pub fn find_dylib_id(
    commands: &[load_command::LoadCommand],
) -> Option<(&LoadCommand, &DylibCommand)> {
    for command in commands {
        if let IdDylib(id_dylib) = &command.command {
            return Some((command, id_dylib));
        }
    }

    None
}
