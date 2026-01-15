use arwen_elf as elf;
use arwen_macho as macho;
use pyo3::{create_exception, exceptions::PyException, PyErr};
use std::error::Error;

#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
pub enum PyMachoError {
    #[error("Macho error: {0}")]
    MachoError(#[from] macho::MachoError),

    #[error("Elf error: {0}")]
    ElfError(#[from] elf::ElfError),
}

fn pretty_print_error(mut err: &dyn Error) -> String {
    let mut result = err.to_string();
    while let Some(source) = err.source() {
        result.push_str(&format!("\nCaused by: {source}"));
        err = source;
    }
    result
}

impl From<PyMachoError> for PyErr {
    fn from(value: PyMachoError) -> Self {
        match value {
            PyMachoError::MachoError(err) => MachoException::new_err(pretty_print_error(&err)),
            PyMachoError::ElfError(err) => ElfException::new_err(pretty_print_error(&err)),
        }
    }
}

create_exception!(exceptions, MachoException, PyException);
create_exception!(exceptions, ElfException, PyException);
