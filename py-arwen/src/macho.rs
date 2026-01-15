use arwen_macho::MachoContainer;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use std::fs;
use std::path::Path;

use crate::error::PyMachoError;

/// Python wrapper for Rust's MachoContainer
#[pyclass]
#[repr(transparent)]
pub struct PyMachoContainer {
    inner: MachoContainer<'static>,
}

#[pymethods]
impl PyMachoContainer {
    /// Create a new MachoContainer by reading a file from a path
    #[staticmethod]
    #[pyo3(text_signature = "(path)")]
    fn from_path(path: &str) -> PyResult<Self> {
        let file_path = Path::new(path);
        let data = fs::read(file_path).map_err(|e| {
            PyErr::new::<PyException, _>(format!("Failed to read file {}: {}", path, e))
        })?;

        // Note: This is safe because we're storing the data in the struct
        let static_data = Box::leak(data.into_boxed_slice());

        let container = MachoContainer::parse(static_data).map_err(PyMachoError::from)?;

        Ok(PyMachoContainer { inner: container })
    }

    /// Add a new rpath to the Mach-O file
    #[pyo3(text_signature = "($self, new_rpath)")]
    fn add_rpath(&mut self, new_rpath: &str) -> PyResult<()> {
        Ok(self
            .inner
            .add_rpath(new_rpath)
            .map_err(PyMachoError::from)?)
    }

    /// Change an existing rpath in the Mach-O file
    #[pyo3(text_signature = "($self, old_rpath, new_rpath)")]
    fn change_rpath(&mut self, old_rpath: &str, new_rpath: &str) -> PyResult<()> {
        Ok(self
            .inner
            .change_rpath(old_rpath, new_rpath)
            .map_err(PyMachoError::from)?)
    }

    /// Remove an existing rpath from the Mach-O file
    #[pyo3(text_signature = "($self, old_rpath)")]
    fn remove_rpath(&mut self, old_rpath: &str) -> PyResult<()> {
        Ok(self
            .inner
            .remove_rpath(old_rpath)
            .map_err(PyMachoError::from)?)
    }

    /// Change the install ID of the Mach-O file (for shared libraries)
    #[pyo3(text_signature = "($self, new_id)")]
    fn change_install_id(&mut self, new_id: &str) -> PyResult<()> {
        Ok(self
            .inner
            .change_install_id(new_id)
            .map_err(PyMachoError::from)?)
    }

    /// Change the install name of a dependency in the Mach-O file
    #[pyo3(text_signature = "($self, old_name, new_name)")]
    fn change_install_name(&mut self, old_name: &str, new_name: &str) -> PyResult<()> {
        Ok(self
            .inner
            .change_install_name(old_name, new_name)
            .map_err(PyMachoError::from)?)
    }

    /// Save the modified Mach-O file to a path
    #[pyo3(text_signature = "($self, path)")]
    fn save(&self, path: &str) -> PyResult<()> {
        fs::write(path, &self.inner.data).map_err(|e| {
            PyErr::new::<PyException, _>(format!("Failed to write file {}: {}", path, e))
        })
    }
}
