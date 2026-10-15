use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use object::elf;

use crate::rewriter::{ElfError, Writer};

pub struct ElfContainer<'a> {
    /// The writer for Elf.
    pub inner: Writer<'a>,

    /// The raw bytes of the ELF file.
    pub data: Vec<u8>,
}

impl<'a> ElfContainer<'a> {
    /// Parse the given bytes and return a new `ElfContainer`.
    pub fn parse(bytes_of_file: &'a [u8]) -> Result<Self, ElfError> {
        let rewriter = Writer::read(bytes_of_file)?;
        Ok(Self {
            inner: rewriter,
            data: bytes_of_file.to_vec(),
        })
    }

    /// Add a runpath to the ELF file.
    pub fn add_runpath(&mut self, new_runpath: impl AsRef<[u8]>) -> Result<(), ElfError> {
        self.inner
            .elf_add_runpath(&[new_runpath.as_ref().to_vec()])?;

        Ok(())
    }

    /// Remove the runpath from the ELF file.
    pub fn remove_runpath(&mut self) -> Result<(), ElfError> {
        self.inner.elf_delete_runpath()?;

        Ok(())
    }

    /// Set runpath to the ELF file.
    pub fn set_runpath(&mut self, set_runpath: impl AsRef<[u8]>) -> Result<(), ElfError> {
        self.inner.elf_set_runpath(set_runpath.as_ref().to_vec())?;

        Ok(())
    }

    /// Print the DT_RUNPATH, or the DT_RPATH if there is no DT_RUNPATH.
    ///
    /// Prints an empty line if there is none, and fails if the file has no
    /// dynamic section.
    pub fn print_runpath(&mut self) -> Result<(), ElfError> {
        if self.inner.builder().dynamic_data().is_none() {
            return Err(ElfError::MissingSection(".dynamic"));
        }
        let runpath = self.inner.elf_runpath().unwrap_or_default();
        println!("{}", String::from_utf8_lossy(runpath));
        Ok(())
    }

    /// Get the RPATH/RUNPATH entries as a list of paths.
    /// Returns entries from DT_RUNPATH if present, otherwise DT_RPATH.
    pub fn get_rpath(&self) -> Vec<String> {
        if let Some(runpath) = self.inner.elf_runpath() {
            let path_str = String::from_utf8_lossy(runpath);
            if path_str.is_empty() {
                return Vec::new();
            }
            path_str.split(':').map(|s| s.to_string()).collect()
        } else {
            Vec::new()
        }
    }

    /// Force the ELF file to use the DT_RPATH instead of DT_RUNPATH.
    pub fn force_rpath(&mut self) -> Result<(), ElfError> {
        self.inner.elf_use_rpath()?;

        Ok(())
    }

    /// Set the PT_INTERPRETER in program header.
    pub fn set_interpreter(&mut self, interpreter: &str) -> Result<(), ElfError> {
        self.inner
            .elf_set_interpreter(interpreter.as_bytes().to_vec())?;

        Ok(())
    }

    /// Print the PT_INTERPRETER program header.
    ///
    /// Fails if the file has no interpreter.
    pub fn print_interpreter(&mut self) -> Result<(), ElfError> {
        let interp = self
            .inner
            .elf_interpreter()
            .ok_or(ElfError::MissingSection(".interp"))?;
        println!("{}", String::from_utf8_lossy(interp));
        Ok(())
    }

    /// Set the OS ABI in the ELF file.
    pub fn set_os_abi(&mut self, os_abi: &str) -> Result<(), ElfError> {
        self.inner.elf_set_osabi(os_abi)?;

        Ok(())
    }

    /// Print the OS ABI in the ELF file.
    pub fn print_os_abi(&mut self) {
        println!("{}", os_abi_name(self.inner.header().os_abi));
    }

    /// Set the SONAME of DT_SONAME.
    pub fn set_soname(&mut self, soname: impl AsRef<[u8]>) -> Result<(), ElfError> {
        self.inner.elf_set_soname(soname.as_ref().to_vec())?;

        Ok(())
    }

    /// Print the SONAME of DT_SONAME.
    pub fn print_soname(&mut self) {
        if let Some(soname) = self.inner.elf_soname() {
            println!("{}", String::from_utf8_lossy(soname));
        }
    }

    /// Remove RPATHs that don't point to the given prefixes.
    pub fn shrink_rpath(&mut self, rpath_prefixes: Vec<String>) -> Result<(), ElfError> {
        self.inner.elf_shrink_rpath(rpath_prefixes)?;

        Ok(())
    }

    /// Add DT_NEEDED to the ELF file.
    pub fn add_needed(&mut self, dt_needed: Vec<impl Into<Vec<u8>>>) -> Result<(), ElfError> {
        let dt_as_u8 = dt_needed
            .into_iter()
            .map(Into::into)
            .collect::<Vec<Vec<u8>>>();
        self.inner.elf_add_needed(&dt_as_u8)?;

        Ok(())
    }

    /// Remove DT_NEEDED from the ELF file.
    pub fn remove_needed(&mut self, dt_needed: Vec<impl Into<Vec<u8>>>) -> Result<(), ElfError> {
        let dt_as_u8 = dt_needed
            .into_iter()
            .map(Into::into)
            .collect::<HashSet<Vec<u8>>>();
        self.inner.elf_delete_needed(&dt_as_u8)?;

        Ok(())
    }

    /// Replace DT_NEEDED in the ELF file.
    pub fn replace_needed(
        &mut self,
        dt_needed: &HashMap<impl Into<Vec<u8>> + Clone, impl Into<Vec<u8>> + Clone>,
    ) -> Result<(), ElfError> {
        let dt_as_u8 = transform_map(dt_needed);
        self.inner.elf_replace_needed(&dt_as_u8)?;

        Ok(())
    }

    /// Print the DT_NEEDED.
    pub fn print_needed(&mut self) {
        for needed in self.inner.elf_needed() {
            println!("{}", String::from_utf8_lossy(needed));
        }
    }

    /// Disable the default library search paths.
    pub fn no_default_lib(&mut self) -> Result<(), ElfError> {
        self.inner.elf_no_default_lib()?;

        Ok(())
    }

    /// Clear the version from given symbol.
    pub fn clear_version_symbol(&mut self, symbol: &str) -> Result<(), ElfError> {
        self.inner.elf_clear_symbol_version(symbol)?;

        Ok(())
    }

    /// Add a debug tag to the ELF file.
    pub fn add_debug_tag(&mut self) -> Result<(), ElfError> {
        self.inner.elf_add_dynamic_debug()?;

        Ok(())
    }

    /// Remove the executable stack execution permission.
    pub fn clear_exec_stack(&mut self) -> Result<(), ElfError> {
        self.inner.elf_clear_exec_stack()?;

        Ok(())
    }

    /// Set the executable stack execution permission.
    pub fn set_exec_stack(&mut self) -> Result<(), ElfError> {
        self.inner.elf_set_exec_stack()?;

        Ok(())
    }

    /// Print the executable stack execution permission:
    /// `execstack: X` if executable, `execstack: -` if not, and
    /// `execstack: ?` if there is no `PT_GNU_STACK` segment.
    pub fn print_exec_stack(&mut self) {
        let state = match self.inner.elf_gnu_exec_stack() {
            Some(flags) if flags & elf::PF_X == elf::PF_X => 'X',
            Some(_) => '-',
            None => '?',
        };
        println!("execstack: {state}");
    }

    /// Rename dynamic symbols in the ELF file.
    pub fn rename_dynamic_symbols(
        &mut self,
        symbols: &HashMap<impl Into<Vec<u8>> + Clone, impl Into<Vec<u8>> + Clone>,
    ) -> Result<(), ElfError> {
        let symbols = transform_map(symbols);

        self.inner.elf_rename_dynamic_symbols(&symbols);

        Ok(())
    }

    /// Set the page size for ELF file segment alignment.
    pub fn set_page_size(&mut self, page_size: u32) -> Result<(), ElfError> {
        self.inner.elf_set_page_size(page_size)?;
        Ok(())
    }

    /// Get the current page size used for segment alignment.
    pub fn get_page_size(&self) -> u32 {
        self.inner.elf_get_page_size()
    }

    pub fn write_to_path(&mut self, path: &Path) -> Result<(), ElfError> {
        self.inner.write_to_path(path)?;
        Ok(())
    }

    pub fn write<W: std::io::Write>(self, w: W) -> Result<(), ElfError> {
        self.inner.write(w)?;
        Ok(())
    }
}

pub fn os_abi_name(os_abi: u8) -> String {
    let name = match os_abi {
        elf::ELFOSABI_SYSV => "System V",
        elf::ELFOSABI_HPUX => "HP-UX",
        elf::ELFOSABI_NETBSD => "NetBSD",
        elf::ELFOSABI_LINUX => "Linux",
        elf::ELFOSABI_HURD => "GNU Hurd",
        elf::ELFOSABI_SOLARIS => "Solaris",
        elf::ELFOSABI_AIX => "AIX",
        elf::ELFOSABI_IRIX => "IRIX",
        elf::ELFOSABI_FREEBSD => "FreeBSD",
        elf::ELFOSABI_TRU64 => "Tru64",
        elf::ELFOSABI_OPENBSD => "OpenBSD",
        elf::ELFOSABI_OPENVMS => "OpenVMS",
        _ => return format!("0x{os_abi:02X}"),
    };
    name.to_string()
}

fn transform_map(
    map: &HashMap<impl Into<Vec<u8>> + Clone, impl Into<Vec<u8>> + Clone>,
) -> HashMap<Vec<u8>, Vec<u8>> {
    map.iter()
        .map(|(k, v)| (k.clone().into(), v.clone().into()))
        .collect()
}
