use std::collections::{HashMap, HashSet};

use object_rewrite::Rewriter;

use crate::macho::MachoError;

pub struct ElfContainer<'a> {
    /// The constructed Mach-O file.
    pub inner: Rewriter<'a>,

    /// The raw bytes of the Mach-O file.
    pub data: Vec<u8>,
}

impl<'a> ElfContainer<'a> {
    /// Parse the given bytes and return a new `MachoContainer`.
    pub fn parse(bytes_of_file: &'a [u8]) -> Result<Self, MachoError> {
        let rewriter = Rewriter::read(bytes_of_file).unwrap();
        Ok(Self {
            inner: rewriter,
            data: bytes_of_file.to_vec(),
        })
    }

    /// Add a runpath to the ELF file.
    pub fn add_runpath(&mut self, new_runpath: &str) -> Result<(), MachoError> {
        let run_path = new_runpath.as_bytes().to_vec();
        self.inner.elf_add_runpath(&[run_path]).unwrap();

        Ok(())
    }

    /// Remove the runpath from the ELF file.
    pub fn remove_runpath(&mut self) -> Result<(), MachoError> {
        self.inner.elf_delete_runpath().unwrap();

        Ok(())
    }

    /// Set runpath to the ELF file.
    pub fn set_runpath(&mut self, set_runpath: &str) -> Result<(), MachoError> {
        let run_path = set_runpath.as_bytes().to_vec();
        self.inner.elf_set_runpath(run_path).unwrap();

        Ok(())
    }

    /// Force the ELF file to use the DT_RPATH instead of DT_RUNPATH.
    pub fn force_rpath(&mut self) -> Result<(), MachoError> {
        self.inner.elf_use_rpath().unwrap();

        Ok(())
    }

    /// Set the PT_INTERPRETER in program header.
    pub fn set_interpreter(&mut self, interpreter: &str) -> Result<(), MachoError> {
        self.inner
            .elf_set_interpreter(interpreter.as_bytes().to_vec())
            .unwrap();

        Ok(())
    }

    /// Set the OS ABI in the ELF file.
    pub fn set_os_abi(&mut self, os_abi: &str) -> Result<(), MachoError> {
        self.inner.elf_set_osabi(os_abi).unwrap();

        Ok(())
    }

    /// Set the SONAME of DT_SONAME.
    pub fn set_soname(&mut self, soname: &str) -> Result<(), MachoError> {
        self.inner
            .elf_set_soname(soname.as_bytes().to_vec())
            .unwrap();

        Ok(())
    }

    pub fn shrink_rpath(&mut self, rpath_prefixes: Vec<String>) -> Result<(), MachoError> {
        self.inner.elf_shrink_rpath(rpath_prefixes).unwrap();

        Ok(())
    }

    pub fn add_needed(&mut self, dt_needed: Vec<String>) -> Result<(), MachoError> {
        let dt_as_u8 = dt_needed
            .iter()
            .map(|x| x.as_bytes().to_vec())
            .collect::<Vec<Vec<u8>>>();
        self.inner.elf_add_needed(&dt_as_u8).unwrap();

        Ok(())
    }

    pub fn remove_needed(&mut self, dt_needed: Vec<String>) -> Result<(), MachoError> {
        let dt_as_u8 = dt_needed
            .iter()
            .map(|x| x.as_bytes().to_vec())
            .collect::<HashSet<Vec<u8>>>();
        self.inner.elf_delete_needed(&dt_as_u8).unwrap();

        Ok(())
    }

    pub fn replace_needed(
        &mut self,
        dt_needed: &HashMap<String, String>,
    ) -> Result<(), MachoError> {
        let dt_as_u8 = transform_map(dt_needed);
        self.inner.elf_replace_needed(&dt_as_u8).unwrap();

        Ok(())
    }

    pub fn no_default_lib(&mut self) -> Result<(), MachoError> {
        self.inner.elf_no_default_lib().unwrap();

        Ok(())
    }

    pub fn clear_version_symbol(&mut self, symbol: &str) -> Result<(), MachoError> {
        self.inner.elf_clear_symbol_version(symbol).unwrap();

        Ok(())
    }

    pub fn add_debug_tag(&mut self) -> Result<(), MachoError> {
        self.inner.elf_add_dynamic_debug().unwrap();

        Ok(())
    }

    pub fn clear_exec_stack(&mut self) -> Result<(), MachoError> {
        self.inner.elf_clear_exec_stack().unwrap();

        Ok(())
    }

    pub fn set_exec_stack(&mut self) -> Result<(), MachoError> {
        self.inner.elf_set_exec_stack().unwrap();

        Ok(())
    }

    pub fn rename_dynamic_symbols(
        &mut self,
        symbols: &HashMap<String, String>,
    ) -> Result<(), MachoError> {
        let symbols = transform_map(symbols);

        self.inner.elf_rename_dynamic_symbols(&symbols);

        Ok(())
    }

    pub fn write<W: std::io::Write>(self, w: W) -> Result<(), MachoError> {
        self.inner.write(w).unwrap();
        Ok(())
    }
}

fn transform_map(map: &HashMap<String, String>) -> HashMap<Vec<u8>, Vec<u8>> {
    map.iter()
        .map(|(k, v)| (k.as_bytes().to_vec(), v.as_bytes().to_vec()))
        .collect()
}
