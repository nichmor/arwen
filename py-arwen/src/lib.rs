use elf::PyElfContainer;
use macho::PyMachoContainer;
use pyo3::prelude::*;
mod elf;
mod error;
mod macho;
mod meta;

use meta::get_arwen_version;

/// A Python module implemented in Rust.
#[pymodule]
fn arwen(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyMachoContainer>()?;
    m.add_class::<PyElfContainer>()?;

    m.add_function(wrap_pyfunction!(get_arwen_version, m).unwrap())?;

    Ok(())
}
