use std::ffi::CStr;

use crate::utils::{align_to_arch, padding_size};

use goblin::{
    container::Ctx,
    mach::load_command::{Dylib, DylibCommand, RpathCommand, LC_RPATH, SIZEOF_RPATH_COMMAND},
};
use scroll::Pwrite;

/// A builder for creating a new `RpathCommand`.
pub struct RpathCommandBuilder {
    raw_str: String,
    ctx: Ctx,
}

impl RpathCommandBuilder {
    /// Creates a new `RpathCommandBuilder` with the given raw string.
    pub fn new(raw_str: &str, ctx: Ctx) -> Self {
        Self {
            raw_str: raw_str.to_string(),
            ctx,
        }
    }

    /// Builds a new `RpathCommand` and returns it along with the raw bytes.
    pub fn build(&self) -> (RpathCommand, Vec<u8>) {
        let raw_c_str_tmplt = format!("{}\0", self.raw_str);
        let raw_c_str = CStr::from_bytes_with_nul(raw_c_str_tmplt.as_bytes()).unwrap();
        let raw_str_size = raw_c_str.count_bytes() + 1;

        // and make it aligned by 4
        eprintln!("raw_str_size: {}", raw_str_size);
        let raw_str_size = padding_size(raw_str_size);

        eprintln!("after aligning raw_str_size: {}", raw_str_size);

        let cmd_size = SIZEOF_RPATH_COMMAND as u32 + raw_str_size as u32;
        let cmd_size = align_to_arch(cmd_size as usize, self.ctx);

        eprintln!("cmd_size: {}", cmd_size);

        let new_rpath = RpathCommand {
            cmd: LC_RPATH,
            cmdsize: cmd_size as u32,
            // path: raw_str_size as u32,
            path: SIZEOF_RPATH_COMMAND as u32,
        };

        // new_rpath

        let mut new_command_buffer = vec![0u8; cmd_size as usize];
        new_command_buffer.fill(0);
        new_command_buffer.pwrite(new_rpath, 0).unwrap();

        eprintln!("raw_c_str: {:?}", raw_c_str.count_bytes());

        new_command_buffer
            .pwrite(raw_c_str, SIZEOF_RPATH_COMMAND)
            .unwrap();

        (new_rpath, new_command_buffer)
    }
}

/// A builder for creating a new `DylibCommand`.
pub struct DlibCommandBuilder {
    dlib_name: String,
    dlib: DylibCommand,
    ctx: Ctx,
}

impl DlibCommandBuilder {
    /// Creates a new `DlibCommandBuilder` with the given dylib name and old dylib command.
    pub fn new(dlib_name: &str, old_dlib: DylibCommand, ctx: Ctx) -> Self {
        Self {
            dlib_name: dlib_name.to_string(),
            dlib: old_dlib,
            ctx,
        }
    }

    /// Builds a new `DylibCommand` and returns it along with the raw bytes.
    pub fn build(&self) -> (DylibCommand, Vec<u8>) {
        let raw_c_str_tmplt = format!("{}\0", self.dlib_name);
        let raw_c_str = CStr::from_bytes_with_nul(raw_c_str_tmplt.as_bytes()).unwrap();
        let raw_str_size = raw_c_str.count_bytes() + 1;

        // and make it aligned by 4
        let raw_str_size = padding_size(raw_str_size);

        // TODO: we should use SIZEOF_DYLIB_COMMAND directly, but it's not in the right size for some reason.
        // it should be 24 instead of 20.
        let cmd_size = 24_u32 + raw_str_size as u32;

        let cmd_size = align_to_arch(cmd_size as usize, self.ctx);

        let dylib = Dylib {
            name: 24_u32,
            timestamp: self.dlib.dylib.timestamp,
            current_version: self.dlib.dylib.current_version,
            compatibility_version: self.dlib.dylib.compatibility_version,
        };

        let new_dylib = DylibCommand {
            cmd: self.dlib.cmd,
            cmdsize: cmd_size as u32,
            // path: raw_str_size as u32,
            // path: SIZEOF_RPATH_COMMAND as u32,
            dylib,
        };

        let mut new_command_buffer = vec![0u8; cmd_size as usize];
        new_command_buffer.fill(0);
        new_command_buffer.pwrite(new_dylib, 0).unwrap();

        new_command_buffer.pwrite(raw_c_str, 24).unwrap();

        (new_dylib, new_command_buffer)
    }
}
