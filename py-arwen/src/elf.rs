use arwen::elf::{ElfContainer, ElfError};
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use crate::error::PyMachoError;

/// Python wrapper for Rust's ElfContainer
#[pyclass]
#[repr(transparent)]
pub struct PyElfContainer {
    inner: ElfContainer<'static>,
}

#[pymethods]
impl PyElfContainer {
    /// Create a new ElfContainer by reading a file from a path
    #[staticmethod]
    #[pyo3(text_signature = "(path)")]
    fn from_path(path: &str) -> PyResult<Self> {
        let file_path = Path::new(path);
        let data = fs::read(file_path).map_err(|e| {
            PyErr::new::<PyException, _>(format!("Failed to read file {}: {}", path, e))
        })?;

        // Note: This is safe because we're storing the data in the struct
        let static_data = Box::leak(data.into_boxed_slice());

        let container = ElfContainer::parse(static_data).map_err(PyMachoError::from)?;

        Ok(PyElfContainer { inner: container })
    }

    /// Add a new runpath to the ELF file
    #[pyo3(text_signature = "($self, new_runpath)")]
    fn add_runpath(&mut self, new_runpath: &str) -> PyResult<()> {
        Ok(self
            .inner
            .add_runpath(new_runpath)
            .map_err(PyMachoError::from)?)
    }

    /// Set runpath in the ELF file
    #[pyo3(text_signature = "($self, set_runpath)")]
    fn set_runpath(&mut self, set_runpath: &str) -> PyResult<()> {
        Ok(self
            .inner
            .set_runpath(set_runpath)
            .map_err(PyMachoError::from)?)
    }

    /// Remove any existing runpath from the ELF file
    #[pyo3(text_signature = "($self)")]
    fn remove_runpath(&mut self) -> PyResult<()> {
        Ok(self.inner.remove_runpath().map_err(PyMachoError::from)?)
    }

    /// Get the current runpath from the ELF file
    #[pyo3(text_signature = "($self)")]
    fn get_runpath(&mut self) -> PyResult<Option<String>> {
        let runpath = self.inner.inner.elf_runpath();
        Ok(runpath.map(|rp| String::from_utf8_lossy(rp).to_string()))
    }

    /// Force the ELF file to use DT_RPATH instead of DT_RUNPATH
    #[pyo3(text_signature = "($self)")]
    fn force_rpath(&mut self) -> PyResult<()> {
        Ok(self.inner.force_rpath().map_err(PyMachoError::from)?)
    }

    /// Shrink rpath to only include specified prefixes
    #[pyo3(text_signature = "($self, rpath_prefixes)")]
    fn shrink_rpath(&mut self, rpath_prefixes: Vec<String>) -> PyResult<()> {
        Ok(self
            .inner
            .shrink_rpath(rpath_prefixes)
            .map_err(PyMachoError::from)?)
    }

    /// Add needed libraries to the ELF file
    #[pyo3(text_signature = "($self, needed_libs)")]
    fn add_needed(&mut self, needed_libs: Vec<String>) -> PyResult<()> {
        Ok(self
            .inner
            .add_needed(needed_libs)
            .map_err(PyMachoError::from)?)
    }

    /// Remove needed libraries from the ELF file
    #[pyo3(text_signature = "($self, needed_libs)")]
    fn remove_needed(&mut self, needed_libs: Vec<String>) -> PyResult<()> {
        Ok(self
            .inner
            .remove_needed(needed_libs)
            .map_err(PyMachoError::from)?)
    }

    /// Replace needed libraries in the ELF file
    #[pyo3(text_signature = "($self, needed_mappings)")]
    fn replace_needed(&mut self, needed_mappings: HashMap<String, String>) -> PyResult<()> {
        Ok(self
            .inner
            .replace_needed(&needed_mappings)
            .map_err(PyMachoError::from)?)
    }

    /// Get the list of needed libraries from the ELF file
    #[pyo3(text_signature = "($self)")]
    fn get_needed(&mut self) -> PyResult<Vec<String>> {
        let needed = self.inner.inner.elf_needed();
        let result = needed
            .map(|n| String::from_utf8_lossy(n).to_string())
            .collect();
        Ok(result)
    }

    /// Change the interpreter path in the ELF file
    #[pyo3(text_signature = "($self, new_interpreter)")]
    fn set_interpreter(&mut self, new_interpreter: &str) -> PyResult<()> {
        Ok(self
            .inner
            .set_interpreter(new_interpreter)
            .map_err(PyMachoError::from)?)
    }

    /// Get the current interpreter from the ELF file
    #[pyo3(text_signature = "($self)")]
    fn get_interpreter(&mut self) -> PyResult<Option<String>> {
        let interp = self.inner.inner.elf_interpreter();
        Ok(interp.map(|i| String::from_utf8_lossy(i).to_string()))
    }

    /// Set the OS ABI in the ELF file
    #[pyo3(text_signature = "($self, os_abi)")]
    fn set_os_abi(&mut self, os_abi: &str) -> PyResult<()> {
        Ok(self.inner.set_os_abi(os_abi).map_err(PyMachoError::from)?)
    }

    /// Get the OS ABI from the ELF file
    #[pyo3(text_signature = "($self)")]
    fn get_os_abi(&mut self) -> PyResult<u8> {
        Ok(self.inner.inner.header().os_abi)
    }

    /// Set the SONAME of the ELF file
    #[pyo3(text_signature = "($self, soname)")]
    fn set_soname(&mut self, soname: &str) -> PyResult<()> {
        Ok(self.inner.set_soname(soname).map_err(PyMachoError::from)?)
    }

    /// Get the SONAME of the ELF file
    #[pyo3(text_signature = "($self)")]
    fn get_soname(&mut self) -> PyResult<Option<String>> {
        let soname = self.inner.inner.elf_soname();
        Ok(soname.map(|s| String::from_utf8_lossy(s).to_string()))
    }

    /// Disable the default library search paths
    #[pyo3(text_signature = "($self)")]
    fn no_default_lib(&mut self) -> PyResult<()> {
        Ok(self.inner.no_default_lib().map_err(PyMachoError::from)?)
    }

    /// Clear a specific symbol version in the ELF file
    #[pyo3(text_signature = "($self, symbol_name)")]
    fn clear_version_symbol(&mut self, symbol_name: &str) -> PyResult<()> {
        Ok(self
            .inner
            .clear_version_symbol(symbol_name)
            .map_err(PyMachoError::from)?)
    }

    /// Add a debug tag to the ELF file
    #[pyo3(text_signature = "($self)")]
    fn add_debug_tag(&mut self) -> PyResult<()> {
        Ok(self.inner.add_debug_tag().map_err(PyMachoError::from)?)
    }

    /// Clear the executable stack flag in the ELF file
    #[pyo3(text_signature = "($self)")]
    fn clear_exec_stack(&mut self) -> PyResult<()> {
        Ok(self.inner.clear_exec_stack().map_err(PyMachoError::from)?)
    }

    /// Set the executable stack flag in the ELF file
    #[pyo3(text_signature = "($self)")]
    fn set_exec_stack(&mut self) -> PyResult<()> {
        Ok(self.inner.set_exec_stack().map_err(PyMachoError::from)?)
    }

    /// Get the executable stack status
    #[pyo3(text_signature = "($self)")]
    fn is_exec_stack(&mut self) -> PyResult<Option<bool>> {
        let result = self
            .inner
            .inner
            .elf_gnu_exec_stack()
            .map(|flag| (flag & object::elf::PF_X) == object::elf::PF_X);
        Ok(result)
    }

    /// Rename dynamic symbols in the ELF file
    #[pyo3(text_signature = "($self, symbols_map)")]
    fn rename_dynamic_symbols(&mut self, symbols_map: HashMap<String, String>) -> PyResult<()> {
        Ok(self
            .inner
            .rename_dynamic_symbols(&symbols_map)
            .map_err(PyMachoError::from)?)
    }

    /// Save the modified ELF file to a path
    #[pyo3(text_signature = "($self, path)")]
    fn save(&mut self, path: &str) -> PyResult<()> {
        Ok(self
            .inner
            .write_to_path(&PathBuf::from(path))
            .map_err(PyMachoError::from)?)
    }
}

// pub(crate) fn register(py: Python<'_>, m: &PyModule) -> PyResult<()> {
//     let submod = PyModule::new(py, "elf")?;

//     // Register the exceptions
//     submod.add("ElfError", py.get_type::<PyElfError>())?;

//     // Register the class
//     submod.add_class::<PyElfContainer>()?;

//     // Add the submodule to the parent module
//     m.add_submodule(submod)?;

//     Ok(())
// }
