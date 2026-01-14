use std::vec;

use goblin::mach::load_command::{self, LoadCommand, RpathCommand};
use goblin::mach::load_command::{CommandVariant::*, DylibCommand};

use scroll::Pwrite;

use crate::{container::HeaderContainer, error::MachoError};

/// Removes a load command from the buffer.
///
/// # Arguments
/// * `buffer` - Mutable byte buffer representing the Mach-O file.
/// * `header` - Header of the macho. It will be updated after removing the load command.
/// * `load_command` - Load Command to remove.
pub fn remove_load_command(
    buffer: &mut Vec<u8>,
    header: &mut HeaderContainer,
    load_command: &LoadCommand,
) -> Result<(), MachoError> {
    // Remove entire command from the buffer
    let drain_offset = load_command.offset + load_command.command.cmdsize();
    buffer.drain(load_command.offset..drain_offset);

    // Update the header
    header.inner.ncmds -= 1;
    header.inner.sizeofcmds -= load_command.command.cmdsize() as u32;

    // Insert padding after the remaining load commands
    let padding_offset = header.size() + header.inner.sizeofcmds as usize;
    let padding_size = load_command.command.cmdsize();

    // Ensure there's enough space for the padding
    if padding_offset + padding_size > buffer.len() {
        buffer.resize(padding_offset + padding_size, 0);
    }

    // Write zero bytes as padding
    let mut zeroing_buffer = vec![0u8; padding_size];
    zeroing_buffer.fill(0);

    let tail = buffer.split_off(padding_offset);

    // Extend with the new slice
    buffer.extend(&zeroing_buffer);

    // Add back the tail
    buffer.extend(tail);

    buffer.pwrite_with(header.inner, 0, header.ctx)?;

    Ok(())
}

/// Insert a new load command at the given offset.
///
/// # Arguments
/// * `buffer` - Mutable byte buffer representing the Mach-O file.
/// * `header` - Header of the macho. It will be updated after removing the load command.
/// * `offset` - Offset to insert the new load command.
/// * `new_cmd_size` - Size of the new command.
/// * `load_command` - Load Command raw data to insert.
pub fn insert_command(
    buffer: &mut Vec<u8>,
    header: &mut HeaderContainer,
    offset: usize,
    new_cmd_size: u32,
    load_data: Vec<u8>,
) -> Result<(), MachoError> {
    // update the header
    header.inner.ncmds += 1;
    header.inner.sizeofcmds += new_cmd_size;

    // write new command
    let tail = buffer.split_off(offset);

    // Extend with the new slice
    buffer.extend(&load_data);

    // Add back the tail
    buffer.extend(tail);

    // We need to drain surplus header pad that is present in macho-file
    // when compiling with -headerpad_max_install_names
    let drain_offset = header.size() + header.inner.sizeofcmds as usize + new_cmd_size as usize;
    buffer.drain(header.size() + header.inner.sizeofcmds as usize..drain_offset);

    buffer.pwrite_with(header.inner, 0, header.ctx)?;

    Ok(())
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
