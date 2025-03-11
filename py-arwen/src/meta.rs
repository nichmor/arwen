use pyo3::pyfunction;

const VERSION: &str = env!("CARGO_PKG_VERSION");
#[pyfunction]
pub fn get_arwen_version() -> &'static str {
    VERSION
}
