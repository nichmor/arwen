use std::{
    collections::{HashMap, HashSet},
    fs, mem,
    path::Path,
};

use object::{
    build::{
        self,
        elf::{Header, VersionId},
        ByteString,
    },
    elf,
    read::elf::FileHeader,
};

use super::{elf::move_sections, BuilderExt, ElfError, Result};

/// A rewriter for object and executable files.
///
/// This struct provides a way to read a file, modify it, and write it back.
#[derive(Debug)]
pub struct Writer<'data> {
    pub(crate) builder: build::elf::Builder<'data>,
    pub(crate) modified: bool,
}

impl<'data> Writer<'data> {
    /// Read a file and create a new rewriter.
    pub fn read(data: &'data [u8]) -> Result<Self> {
        let builder = build::elf::Builder::read(data).map_err(ElfError::Parse)?;
        Ok(Self {
            builder,
            modified: false,
        })
    }

    /// Returns the underlying `object::elf::Builder` instance.
    /// This is useful for inspections of the ELF file.
    ///
    /// All modifications should be done through the methods provided by Writer itself.
    pub fn builder(&self) -> &build::elf::Builder<'data> {
        &self.builder
    }

    /// Delete symbols from the symbol table.
    pub fn elf_delete_symbols(&mut self, names: &HashSet<Vec<u8>>) {
        for symbol in &mut self.builder.dynamic_symbols {
            if names.contains(&*symbol.name) {
                symbol.delete = true;
                self.modified = true;
            }
        }
    }

    /// Delete symbols from the dynamic symbol table.
    pub fn elf_delete_dynamic_symbols(&mut self, names: &HashSet<Vec<u8>>) {
        for symbol in &mut self.builder.symbols {
            if names.contains(&*symbol.name) {
                symbol.delete = true;
                self.modified = true;
            }
        }
    }

    /// Rename symbols in the symbol table.
    ///
    /// The `names` map is from old names to new names.
    pub fn elf_rename_symbols(&mut self, names: &HashMap<Vec<u8>, Vec<u8>>) {
        for symbol in &mut self.builder.dynamic_symbols {
            if let Some(name) = names.get(&*symbol.name) {
                let name = name.clone().into();
                symbol.name = name;
                self.modified = true;
            }
        }
    }

    /// Rename symbols in the dynamic symbol table.
    ///
    /// The `names` map is from old names to new names.
    pub fn elf_rename_dynamic_symbols(&mut self, names: &HashMap<Vec<u8>, Vec<u8>>) {
        for symbol in &mut self.builder.dynamic_symbols {
            if let Some(name) = names.get(&*symbol.name) {
                let name = name.clone().into();
                symbol.name = name;
                self.modified = true;
            }
        }
    }

    pub(crate) fn elf_delete_sections(&mut self, names: &HashSet<Vec<u8>>) {
        for section in &mut self.builder.sections {
            if names.contains(&*section.name) {
                // Associated program header will be deleted by delete_orphan_segments.
                section.delete = true;
                self.modified = true;
            }
        }
    }

    pub(crate) fn elf_rename_sections(&mut self, names: &HashMap<Vec<u8>, Vec<u8>>) {
        for section in &mut self.builder.sections {
            if let Some(name) = names.get(&*section.name) {
                let name = name.clone().into();
                section.name = name;
                self.modified = true;
            }
        }
    }

    /// Add a `DT_DEBUG` entry to the dynamic section.
    pub fn elf_add_dynamic_debug(&mut self) -> Result<()> {
        let dynamic = self.builder.dynamic_data_mut().ok_or_else(|| {
            ElfError::Modify("No dynamic section found; can't add debug entry".to_string())
        })?;
        if dynamic.iter().any(|entry| entry.tag() == elf::DT_DEBUG) {
            return Ok(());
        }
        dynamic.push(build::elf::Dynamic::Integer {
            tag: elf::DT_DEBUG,
            val: 0,
        });
        self.modified = true;
        Ok(())
    }

    /// Find the first `DT_RUNPATH` or `DT_RPATH` entry in the dynamic section.
    pub fn elf_runpath(&self) -> Option<&[u8]> {
        let dynamic = self.builder.dynamic_data()?;
        for entry in dynamic.iter() {
            let build::elf::Dynamic::String { tag, val } = entry else {
                continue;
            };
            if *tag != elf::DT_RPATH && *tag != elf::DT_RUNPATH {
                continue;
            }
            return Some(val);
        }
        None
    }

    /// Delete any `DT_RUNPATH` or `DT_RPATH` entries in the dynamic section.
    pub fn elf_delete_runpath(&mut self) -> Result<()> {
        let dynamic = self.builder.dynamic_data_mut().ok_or_else(|| {
            ElfError::Modify("No dynamic section found; can't delete runpath".to_string())
        })?;
        let mut modified = false;
        dynamic.retain(|entry| {
            let tag = entry.tag();
            if tag != elf::DT_RPATH && tag != elf::DT_RUNPATH {
                return true;
            }

            modified = true;
            false
        });
        if modified {
            self.modified = true;
        }
        Ok(())
    }

    /// Set the path for any `DT_RUNPATH` or `DT_RPATH` entry in the dynamic section.
    pub fn elf_set_runpath(&mut self, runpath: Vec<u8>) -> Result<()> {
        let dynamic = self.builder.dynamic_data_mut().ok_or_else(|| {
            ElfError::Modify("No dynamic section found; can't set runpath".to_string())
        })?;
        let mut found = false;
        for entry in dynamic.iter_mut() {
            let build::elf::Dynamic::String { tag, val } = entry else {
                continue;
            };
            if *tag != elf::DT_RPATH && *tag != elf::DT_RUNPATH {
                continue;
            }

            *val = build::ByteString::from(runpath.clone());
            found = true;
        }
        if !found {
            let val = build::ByteString::from(runpath);
            dynamic.push(build::elf::Dynamic::String {
                tag: elf::DT_RUNPATH,
                val,
            });
        }
        self.modified = true;
        Ok(())
    }

    /// Add additional paths to any `DT_RUNPATH` or `DT_RPATH` entry in the dynamic section.
    pub fn elf_add_runpath(&mut self, runpaths: &[Vec<u8>]) -> Result<()> {
        let dynamic = self.builder.dynamic_data_mut().ok_or_else(|| {
            ElfError::Modify("No dynamic section found; can't add runpath".to_string())
        })?;
        let mut found = false;
        for entry in dynamic.iter_mut() {
            let build::elf::Dynamic::String { tag, val } = entry else {
                continue;
            };
            if *tag != elf::DT_RPATH && *tag != elf::DT_RUNPATH {
                continue;
            }

            for path in runpaths {
                if !val.is_empty() {
                    val.to_mut().push(b':');
                }
                val.to_mut().extend_from_slice(path);
            }
            found = true;
        }
        if !found {
            let val = runpaths.join(&[b':'][..]).into();
            dynamic.push(build::elf::Dynamic::String {
                tag: elf::DT_RUNPATH,
                val,
            });
        }
        self.modified = true;
        Ok(())
    }

    /// Change any `DT_RPATH` entry in the dynamic section to `DT_RUNPATH`.
    pub fn elf_use_runpath(&mut self) -> Result<()> {
        let dynamic = self.builder.dynamic_data_mut().ok_or_else(|| {
            ElfError::Modify("No dynamic section found; can't change runpath".to_string())
        })?;
        for entry in dynamic.iter_mut() {
            let build::elf::Dynamic::String { tag, .. } = entry else {
                continue;
            };
            if *tag != elf::DT_RPATH {
                continue;
            }
            *tag = elf::DT_RUNPATH;
            self.modified = true;
        }
        Ok(())
    }

    /// Change any `DT_RUNPATH` entry in the dynamic section to `DT_RPATH`.
    pub fn elf_use_rpath(&mut self) -> Result<()> {
        let dynamic = self.builder.dynamic_data_mut().ok_or_else(|| {
            ElfError::Modify("No dynamic section found; can't change rpath".to_string())
        })?;
        for entry in dynamic.iter_mut() {
            let build::elf::Dynamic::String { tag, .. } = entry else {
                continue;
            };
            if *tag != elf::DT_RUNPATH {
                continue;
            }
            *tag = elf::DT_RPATH;
            self.modified = true;
        }
        Ok(())
    }

    /// Find the `DT_NEEDED` entries in the dynamic section.
    pub fn elf_needed(&self) -> impl Iterator<Item = &[u8]> {
        let dynamic = self.builder.dynamic_data().unwrap_or(&[]);
        dynamic.iter().filter_map(|entry| {
            if let build::elf::Dynamic::String { tag, val } = entry {
                if *tag == elf::DT_NEEDED {
                    return Some(val.as_slice());
                }
            }
            None
        })
    }

    /// Delete `DT_NEEDED` entries from the dynamic section.
    pub fn elf_delete_needed(&mut self, names: &HashSet<Vec<u8>>) -> Result<()> {
        let dynamic = self.builder.dynamic_data_mut().ok_or_else(|| {
            ElfError::Modify("No dynamic section found; can't delete needed library".to_string())
        })?;
        let mut modified = false;
        dynamic.retain(|entry| {
            let build::elf::Dynamic::String { tag, val } = entry else {
                return true;
            };
            if *tag != elf::DT_NEEDED || !names.contains(val.as_slice()) {
                return true;
            }
            modified = true;
            false
        });
        if modified {
            self.modified = true;
        }
        Ok(())
    }

    /// Replace `DT_NEEDED` entries in the dynamic section.
    pub fn elf_replace_needed(&mut self, names: &HashMap<Vec<u8>, Vec<u8>>) -> Result<()> {
        let dynamic = self.builder.dynamic_data_mut().ok_or_else(|| {
            ElfError::Modify("No dynamic section found; can't replace needed library".to_string())
        })?;
        for entry in dynamic.iter_mut() {
            let build::elf::Dynamic::String { tag, val } = entry else {
                continue;
            };
            if *tag != elf::DT_NEEDED {
                continue;
            }
            let Some(name) = names.get(val.as_slice()) else {
                continue;
            };

            let name = name.clone().into();
            *val = name;
            self.modified = true;
        }
        Ok(())
    }

    /// Add `DT_NEEDED` entries to the start of the dynamic section.
    ///
    /// This does not add a `DT_NEEDED` entry if the library is already listed.
    pub fn elf_add_needed(&mut self, names: &[Vec<u8>]) -> Result<()> {
        let dynamic = self.builder.dynamic_data_mut().ok_or_else(|| {
            ElfError::Modify("No dynamic section found; can't add needed library".to_string())
        })?;
        let mut found = HashSet::new();
        for entry in dynamic.iter() {
            let build::elf::Dynamic::String { tag, val } = entry else {
                continue;
            };
            if *tag != elf::DT_NEEDED {
                continue;
            }
            found.insert(val.clone());
        }
        for name in names
            .iter()
            .rev()
            .filter(|name| !found.contains(name.as_slice()))
        {
            let val = name.clone().into();
            dynamic.insert(
                0,
                build::elf::Dynamic::String {
                    tag: elf::DT_NEEDED,
                    val,
                },
            );
            self.modified = true;
        }
        Ok(())
    }

    /// Find the `DT_SONAME` entry in the dynamic section.
    pub fn elf_soname(&self) -> Option<&[u8]> {
        let id = self.builder.dynamic_section()?;
        let section = self.builder.sections.get(id);
        let build::elf::SectionData::Dynamic(dynamic) = &section.data else {
            return None;
        };
        for entry in dynamic.iter() {
            let build::elf::Dynamic::String { tag, val } = entry else {
                continue;
            };
            if *tag != elf::DT_SONAME {
                continue;
            }

            return Some(val);
        }
        None
    }

    /// Set the `DT_SONAME` entry in the dynamic section.
    pub fn elf_set_soname(&mut self, soname: Vec<u8>) -> Result<()> {
        let dynamic = self.builder.dynamic_data_mut().ok_or_else(|| {
            ElfError::Modify("No dynamic section found; can't set soname".to_string())
        })?;
        let mut found = false;
        for entry in dynamic.iter_mut() {
            let build::elf::Dynamic::String { tag, val } = entry else {
                continue;
            };
            if *tag != elf::DT_SONAME {
                continue;
            }

            *val = soname.clone().into();
            found = true;
        }
        if !found {
            let val = soname.into();
            dynamic.push(build::elf::Dynamic::String {
                tag: elf::DT_SONAME,
                val,
            });
        }
        self.modified = true;
        Ok(())
    }

    /// Find the interpreter path in the `PT_INTERP` segment.
    pub fn elf_interpreter(&self) -> Option<&[u8]> {
        self.builder.interp_data()
    }

    /// Set the interpreter path in the `PT_INTERP` segment.
    ///
    /// The null terminator is automatically added if needed.
    pub fn elf_set_interpreter(&mut self, mut interpreter: Vec<u8>) -> Result<()> {
        let data = self.builder.interp_data_mut().ok_or_else(|| {
            ElfError::Modify("No interp section found; can't set interpreter".to_string())
        })?;

        if !interpreter.is_empty() && interpreter.last() != Some(&0) {
            interpreter.push(0);
        }
        *data = interpreter.into();
        self.modified = true;
        Ok(())
    }

    /// Set the EI_OSABI to some ABI
    pub fn elf_set_osabi(&mut self, abi_name: &str) -> Result<()> {
        let header = &mut self.builder.header;

        // lowercase the name
        let abi_name = abi_name.to_lowercase();

        let new_abi = match abi_name.as_str() {
            "sysv" => elf::ELFOSABI_SYSV,
            "hpux" => elf::ELFOSABI_HPUX,
            "netbsd" => elf::ELFOSABI_NETBSD,
            "linux" => elf::ELFOSABI_LINUX,
            "hurd" | "gnu-hurd" | "gnu hurd" => elf::ELFOSABI_HURD,
            "solaris" => elf::ELFOSABI_SOLARIS,
            "aix" => elf::ELFOSABI_AIX,
            "irix" => elf::ELFOSABI_IRIX,
            "freebsd" => elf::ELFOSABI_FREEBSD,
            "tru64" => elf::ELFOSABI_TRU64,
            "modesto" => elf::ELFOSABI_MODESTO,
            "openbsd" => elf::ELFOSABI_OPENBSD,
            "openvms" => elf::ELFOSABI_OPENVMS,
            "nsk" => elf::ELFOSABI_NSK,
            "aros" => elf::ELFOSABI_AROS,
            "fenixos" => elf::ELFOSABI_FENIXOS,
            "cloudabi" => elf::ELFOSABI_CLOUDABI,
            _ => return Err(ElfError::Modify(format!("Unknown ABI {abi_name}"))),
        };

        header.os_abi = new_abi;

        self.modified = true;
        Ok(())
    }

    /// Remove from the DT_RUNPATH or DT_RPATH all directories that do not contain a library referenced by DT_NEEDED.
    pub fn elf_shrink_rpath(&mut self, allowed_rpath_prefixes: Vec<String>) -> Result<()> {
        let endian = self.builder.endian;
        let e_machine = self.builder.header.e_machine;
        let needed_libraries = self
            .elf_needed()
            .map(|e| e.to_vec())
            .collect::<HashSet<_>>();
        let dynamic = self.builder.dynamic_data_mut().ok_or_else(|| {
            ElfError::Modify("No dynamic section found; can't shrink rpath".to_string())
        })?;
        let mut found = false;
        for entry in dynamic.iter_mut() {
            let build::elf::Dynamic::String { tag, val } = entry else {
                continue;
            };
            if *tag != elf::DT_RPATH && *tag != elf::DT_RUNPATH {
                continue;
            }

            found = true;

            let rpath = val.clone();
            let rpath_str = rpath.to_string();
            let mut rpath_vec: Vec<&str> = rpath_str.split(":").collect();
            // rpath_vec.retain(|&x| x != "");

            // Check if each directory contains a library referenced by DT_NEEDED
            rpath_vec.retain(|&dir| {
                let dir_path = Path::new(dir);

                // Keep non-absolute paths (e.g., "$ORIGIN")
                if !dir.starts_with('/') {
                    return true;
                };

                // Check allowed prefixes
                if !allowed_rpath_prefixes.is_empty()
                    && !allowed_rpath_prefixes
                        .iter()
                        .any(|prefix| dir.starts_with(prefix))
                {
                    eprintln!("Removing {dir} from RPATH due to non-allowed prefix");
                    return false;
                }

                for lib_needed in needed_libraries.iter() {
                    let possible_library = String::from_utf8(lib_needed.to_vec()).unwrap();

                    let library_location = dir_path.join(possible_library);
                    // read the elf file
                    let file_content = fs::read(library_location);
                    let Ok(file_content) = file_content else {
                        continue;
                    };
                    let elf_file =
                        elf::FileHeader64::<object::Endianness>::parse(file_content.as_slice())
                            .unwrap();
                    let lib_e_machine = elf_file.e_machine(endian);

                    if lib_e_machine == e_machine {
                        return true;
                    }
                }
                eprintln!("Removing directory {dir}");
                false
            });

            eprintln!("New rpath: {rpath_vec:?}");

            let new_rpath = rpath_vec.join(":");
            *val.to_mut() = ByteString::from(new_rpath.as_bytes()).to_vec();
        }
        if !found {
            return Err(ElfError::Modify("No DT_RPATH entry found".to_string()));
        }
        self.modified = true;
        Ok(())
    }

    /// Disable the default library search paths
    pub fn elf_no_default_lib(&mut self) -> Result<()> {
        let dynamic = self.builder.dynamic_data_mut().ok_or_else(|| {
            ElfError::Modify("No dynamic section found; can't set soname".to_string())
        })?;
        let mut found = false;
        for entry in dynamic.iter_mut() {
            let build::elf::Dynamic::Integer { tag, val } = entry else {
                continue;
            };
            if *tag != elf::DT_FLAGS_1 {
                continue;
            }

            *val |= elf::DF_1_NODEFLIB as u64;
            found = true;
        }
        if !found {
            dynamic.push(build::elf::Dynamic::Integer {
                tag: elf::DT_FLAGS_1,
                val: elf::DF_1_NODEFLIB as u64,
            });
        }
        self.modified = true;
        Ok(())
    }

    /// Clear the symbol version information for a symbol
    pub fn elf_clear_symbol_version(&mut self, symbol: &str) -> Result<()> {
        let symbols = &mut self.builder.dynamic_symbols;

        let mut found = false;
        for entry in symbols.iter_mut() {
            // verify if the symbol is the one we are looking for
            if entry.name == symbol.into() {
                // clear the version
                entry.version = VersionId::global();
                found = true;
            }
        }
        if found {
            self.modified = true;
        }
        Ok(())
    }

    /// Clear the symbol version information for a symbol
    pub fn elf_clear_exec_stack(&mut self) -> Result<()> {
        let gnu_stack = self.builder.gnu_stack_mut();

        let gnu_stack = if let Some(segment) = gnu_stack {
            segment
        } else {
            // add a new PT_GNU_STACK segment
            self.builder.add_gnu_stack()
        };

        gnu_stack.p_flags &= !elf::PF_X;

        self.modified = true;

        Ok(())
    }

    /// Clear the symbol version information for a symbol
    pub fn elf_set_exec_stack(&mut self) -> Result<()> {
        let gnu_stack = self.builder.gnu_stack_mut();

        let gnu_stack = if let Some(segment) = gnu_stack {
            segment
        } else {
            // add a new PT_GNU_STACK segment
            self.builder.add_gnu_stack()
        };

        gnu_stack.p_flags |= elf::PF_X;

        self.modified = true;

        Ok(())
    }

    /// Return the flags of the PT_GNU_STACK segment
    pub fn elf_gnu_exec_stack(&self) -> Option<u32> {
        let gnu_stack = self.builder.gnu_stack();
        if let Some(segment) = gnu_stack {
            return Some(segment.p_flags);
        }
        None
    }

    /// Return the ELF file header
    pub fn header(&self) -> &Header {
        &self.builder.header
    }

    pub(crate) fn elf_finalize(&mut self) -> Result<()> {
        if self.modified {
            move_sections(&mut self.builder)?;
        }
        Ok(())
    }

    /// Write the file to an output stream.
    pub fn write<W: std::io::Write>(mut self, w: W) -> Result<()> {
        self.elf_finalize()?;
        let mut buffer = object::write::StreamingBuffer::new(w);
        self.builder.write(&mut buffer).map_err(ElfError::Write)?;
        buffer.result().map_err(ElfError::Io)
    }

    /// Write the builded/patched ELF file to a path.
    pub fn write_to_path(&mut self, path: &Path) -> Result<()> {
        let file_writer = fs::File::create(path).map_err(ElfError::Io)?;

        self.elf_finalize()?;
        let mut buffer = object::write::StreamingBuffer::new(file_writer);
        let new_builder = build::elf::Builder::new(self.builder.endian, self.builder.is_64);

        let builder = mem::replace(&mut self.builder, new_builder);

        builder.write(&mut buffer).map_err(ElfError::Write)?;
        buffer.result().map_err(ElfError::Io)
    }

    /// Delete symbols from the symbol table.
    ///
    /// For ELF files, this deletes symbols from both the symbol table and the
    /// dynamic symbol table.
    pub fn delete_symbols(&mut self, names: &HashSet<Vec<u8>>) {
        self.elf_delete_symbols(names);
        self.elf_delete_dynamic_symbols(names);
    }

    /// Rename symbols in the symbol table.
    ///
    /// For ELF files, this renames symbols in both the symbol table and the
    /// dynamic symbol table.
    ///
    /// The `names` map is from old names to new names.
    pub fn rename_symbols(&mut self, names: &HashMap<Vec<u8>, Vec<u8>>) {
        self.elf_rename_symbols(names);
        self.elf_rename_dynamic_symbols(names);
    }

    /// Delete sections from the file.
    pub fn delete_sections(&mut self, names: &HashSet<Vec<u8>>) {
        self.elf_delete_sections(names);
    }

    /// Rename sections in the file.
    ///
    /// The `names` map is from old names to new names.
    pub fn rename_sections(&mut self, names: &HashMap<Vec<u8>, Vec<u8>>) {
        self.elf_rename_sections(names);
    }
}
