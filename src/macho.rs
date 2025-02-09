use core::panic;

use goblin::{
    container,
    mach::{
        fat::{self, FatArch},
        header::{Header, SIZEOF_HEADER_32, SIZEOF_HEADER_64},
        parse_magic_and_ctx, peek, MachO, MultiArch, SingleArch,
    },
};

use crate::{
    commands::{DlibCommandBuilder, RpathCommandBuilder},
    patcher::{
        find_dylib_command, find_dylib_id, find_rpath_command, insert_command, remove_load_command,
    },
};

pub struct SingleMachO<'a> {
    /// The parsed Mach-O file.
    pub inner: MachO<'a>,

    /// The raw bytes of the Mach-O file.
    pub data: Vec<u8>,

    /// The context of the container.
    /// This is used to determine the architecture of the Mach-O file.
    pub ctx: container::Ctx,
}

impl SingleMachO<'_> {
    pub fn add_rpath(&mut self, new_rpath: &str) {
        let mut header = HeaderContainer::new(self.inner.header, self.ctx);

        let (new_rpath, new_rpath_command_buffer) =
            RpathCommandBuilder::new(new_rpath, self.ctx).build();

        let offset_size = header.size() + header.inner.sizeofcmds as usize;

        insert_command(
            &mut self.data,
            &mut header,
            offset_size,
            new_rpath.cmdsize,
            new_rpath_command_buffer,
        );
    }

    pub fn change_rpath(&mut self, old_rpath: &str, new_rpath: &str) {
        let mut header = HeaderContainer::new(self.inner.header, self.ctx);

        let old_rpath_index = self
            .inner
            .rpaths
            .iter()
            .position(|rpath| rpath == &old_rpath)
            .expect("rpath should exist");

        // now based on the index, we need to find the RpathCommand from the load commands
        let (load_command, _rpath_command) =
            find_rpath_command(&self.inner.load_commands, old_rpath_index)
                .expect("rpath command should exist");

        remove_load_command(&mut self.data, &mut header, load_command);

        let (new_rpath, new_rpath_command_buffer) =
            RpathCommandBuilder::new(new_rpath, self.ctx).build();

        insert_command(
            &mut self.data,
            &mut header,
            load_command.offset,
            new_rpath.cmdsize,
            new_rpath_command_buffer,
        );
    }

    pub fn remove_rpath(&mut self, old_rpath: &str) {
        let mut header = HeaderContainer::new(self.inner.header, self.ctx);

        let old_rpath_index = self
            .inner
            .rpaths
            .iter()
            .position(|rpath| rpath == &old_rpath)
            .expect("rpath should exist");

        // now based on the index, we need to find the RpathCommand from the load commands
        let (load_command, _rpath_command) =
            find_rpath_command(&self.inner.load_commands, old_rpath_index)
                .expect("rpath command should exist");

        remove_load_command(&mut self.data, &mut header, load_command);
    }

    pub fn change_install_id(&mut self, new_id: &str) {
        let mut header = HeaderContainer::new(self.inner.header, self.ctx);

        // now based on the index, we need to find the RpathCommand from the load commands
        let (load_command, old_dylib) = find_dylib_id(&self.inner.load_commands)
            .expect("LC_ID_DYLIB is missing or file is not a shared library");

        remove_load_command(&mut self.data, &mut header, load_command);

        let (new_dylib, new_dylib_command_buffer) =
            DlibCommandBuilder::new(&new_id, *old_dylib, self.ctx).build();

        insert_command(
            &mut self.data,
            &mut header,
            load_command.offset,
            new_dylib.cmdsize,
            new_dylib_command_buffer,
        );
    }

    pub fn change_install_name(&mut self, old_name: &str, new_name: &str) {
        let mut header = HeaderContainer::new(self.inner.header, self.ctx);

        // let's start with load commands
        let old_dylib = self
            .inner
            .libs
            .iter()
            .position(|name| name == &old_name)
            .expect("dylib name should exist");

        // now based on the index, we need to find the RpathCommand from the load commands
        // we use -1 because dylib contains self, so we need to omit it
        let (load_command, old_dylib) =
            find_dylib_command(&self.inner.load_commands, old_dylib - 1)
                .expect("rpath command should exist");

        remove_load_command(&mut self.data, &mut header, load_command);

        let (new_dylib, new_dylib_command_buffer) =
            DlibCommandBuilder::new(&new_name, *old_dylib, self.ctx).build();

        insert_command(
            &mut self.data,
            &mut header,
            load_command.offset,
            new_dylib.cmdsize,
            new_dylib_command_buffer,
        );
    }
}

pub struct FatMacho<'a> {
    pub inner: SingleMachO<'a>,

    // _internal: SingleArch<'a>,
    pub arch: FatArch,
}

pub struct FatMachoContainer<'a> {
    /// The parsed Mach-O file.
    pub archs: Vec<FatMacho<'a>>,

    /// Data of all the Mach-O files
    pub data: Vec<u8>,
}

pub enum MachoType<'a> {
    SingleArch(SingleMachO<'a>),
    Fat(FatMachoContainer<'a>),
}

pub struct MachoContainer<'a> {
    /// The constructed Mach-O file.
    pub inner: MachoType<'a>,

    /// The raw bytes of the Mach-O file.
    pub data: Vec<u8>,
}

impl<'a> MachoContainer<'a> {
    pub fn add_rpath(&mut self, new_rpath: &str) {
        match &mut self.inner {
            MachoType::SingleArch(single) => {
                single.add_rpath(new_rpath);

                // save back changed data
                // TODO: think how to overcome clonning again
                self.data = single.data.clone();
            }
            MachoType::Fat(fat) => {
                for macho in &mut fat.archs {
                    macho.inner.add_rpath(new_rpath);

                    // save back changed data
                    // by writing one piece of macho back into the archive

                    let arch = macho.arch;
                    self.data.splice(
                        arch.offset as usize..arch.offset as usize + arch.size as usize,
                        macho.inner.data.clone(),
                    );
                }
            }
        }
    }

    pub fn change_rpath(&mut self, old_rpath: &str, new_rpath: &str) {
        match &mut self.inner {
            MachoType::SingleArch(single) => {
                single.change_rpath(old_rpath, new_rpath);

                // save back changed data
                // TODO: think how to overcome clonning again
                self.data = single.data.clone();
            }
            MachoType::Fat(fat) => {
                for macho in &mut fat.archs {
                    macho.inner.change_rpath(old_rpath, new_rpath);

                    // save back changed data
                    // by writing one piece of macho back into the archive
                    let arch = macho.arch;
                    self.data.splice(
                        arch.offset as usize..arch.offset as usize + arch.size as usize,
                        macho.inner.data.clone(),
                    );
                }
            }
        }
    }

    pub fn remove_rpath(&mut self, old_rpath: &str) {
        match &mut self.inner {
            MachoType::SingleArch(single) => {
                single.remove_rpath(old_rpath);

                // save back changed data
                // TODO: think how to overcome clonning again
                self.data = single.data.clone();
            }
            MachoType::Fat(fat) => {
                for macho in &mut fat.archs {
                    macho.inner.remove_rpath(old_rpath);

                    // save back changed data
                    // by writing one piece of macho back into the archive

                    let arch = macho.arch;
                    self.data.splice(
                        arch.offset as usize..arch.offset as usize + arch.size as usize,
                        macho.inner.data.clone(),
                    );
                }
            }
        }
    }

    pub fn change_install_id(&mut self, new_id: &str) {
        match &mut self.inner {
            MachoType::SingleArch(single) => {
                single.change_install_id(new_id);

                // save back changed data
                // TODO: think how to overcome clonning again
                self.data = single.data.clone();
            }
            MachoType::Fat(fat) => {
                for macho in &mut fat.archs {
                    macho.inner.change_install_id(new_id);

                    // save back changed data
                    // by writing one piece of macho back into the archive

                    let arch = macho.arch;
                    self.data.splice(
                        arch.offset as usize..arch.offset as usize + arch.size as usize,
                        macho.inner.data.clone(),
                    );
                }
            }
        }
    }

    pub fn change_install_name(&mut self, old_name: &str, new_name: &str) {
        match &mut self.inner {
            MachoType::SingleArch(single) => {
                single.change_install_name(old_name, new_name);

                // save back changed data
                // TODO: think how to overcome clonning again
                self.data = single.data.clone();
            }
            MachoType::Fat(fat) => {
                for macho in &mut fat.archs {
                    macho.inner.change_install_name(old_name, new_name);

                    // save back changed data
                    // by writing one piece of macho back into the archive

                    let arch = macho.arch;
                    self.data.splice(
                        arch.offset as usize..arch.offset as usize + arch.size as usize,
                        macho.inner.data.clone(),
                    );
                }
            }
        }
    }
}

impl<'a> MachoContainer<'a> {
    pub fn parse(bytes_of_file: &'a [u8]) -> Self {
        // Using goblin MachO parser directly
        // is not possible, as the wrapper type request to get the reference for the bytes
        // we, on other hand, need to own the bytes
        // so we simplify the complexity by parsing the Mach-O file ourselves
        let magic = peek(bytes_of_file, 0).unwrap();
        match magic {
            fat::FAT_MAGIC => {
                let multi_arch = MultiArch::new(bytes_of_file).unwrap();

                let mut archs = Vec::new();
                for arch in multi_arch.iter_arches() {
                    let arch = arch.unwrap();
                    archs.push(arch);
                }

                let mut machos = Vec::new();
                for (idx, arch) in multi_arch.into_iter().enumerate() {
                    let single = arch.unwrap();
                    let SingleArch::MachO(mach_o) = single else {
                        panic!("Could not parse the Mach-O file");
                    };

                    let fat_arch = archs.get(idx).unwrap();

                    let data = fat_arch.slice(bytes_of_file);

                    // it's the duplicate from beyond
                    let (_, maybe_ctx) = parse_magic_and_ctx(data, 0).unwrap();
                    let ctx = if let Some(ctx) = maybe_ctx {
                        ctx
                    } else {
                        panic!("Could not determine the architecture of the Mach-O file");
                    };

                    let single_mach = SingleMachO {
                        inner: mach_o,
                        data: data.to_vec(),
                        ctx,
                    };

                    let fat_macho = FatMacho {
                        inner: single_mach,
                        arch: *fat_arch,
                    };

                    // let single_arch_data = FatMacho
                    machos.push(fat_macho);
                }

                let container = FatMachoContainer {
                    archs: machos,
                    data: bytes_of_file.to_vec(),
                };
                MachoContainer {
                    inner: MachoType::Fat(container),
                    data: bytes_of_file.to_vec(),
                }
            }
            _ => {
                let mach_o = MachO::parse(bytes_of_file, 0).unwrap();

                let (_, maybe_ctx) = parse_magic_and_ctx(bytes_of_file, 0).unwrap();
                let ctx = if let Some(ctx) = maybe_ctx {
                    ctx
                } else {
                    panic!("Could not determine the architecture of the Mach-O file");
                };

                let mach_type = MachoType::SingleArch(SingleMachO {
                    inner: mach_o,
                    data: bytes_of_file.to_vec(),
                    ctx,
                });

                MachoContainer {
                    inner: mach_type,
                    data: bytes_of_file.to_vec(),
                }
            }
        }
    }
}

pub struct HeaderContainer {
    pub inner: Header,
    pub ctx: container::Ctx,
}

impl HeaderContainer {
    pub fn new(header: Header, ctx: container::Ctx) -> Self {
        HeaderContainer { inner: header, ctx }
    }

    pub fn size(&self) -> usize {
        if self.ctx.container.is_big() {
            SIZEOF_HEADER_64
        } else {
            SIZEOF_HEADER_32
        }
    }
}
