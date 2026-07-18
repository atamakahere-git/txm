use libtxm;
use pyo3::prelude::*;

#[pyfunction]
fn render(input: &str) -> PyResult<String> {
    libtxm::render(input).map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
}

#[pymodule]
fn txm(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(render, m)?)?;
    Ok(())
}
