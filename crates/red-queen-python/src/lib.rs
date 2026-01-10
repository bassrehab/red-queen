//! Python bindings for Red Queen.

use pyo3::prelude::*;

/// Red Queen Python module.
#[pymodule]
fn red_queen(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyRedQueen>()?;
    Ok(())
}

/// Main Red Queen interface for Python.
#[pyclass]
struct PyRedQueen {
    population_size: usize,
}

#[pymethods]
impl PyRedQueen {
    #[new]
    fn new() -> Self {
        Self {
            population_size: 100,
        }
    }

    /// Set population size.
    fn with_population_size(mut slf: PyRefMut<'_, Self>, size: usize) -> PyRefMut<'_, Self> {
        slf.population_size = size;
        slf
    }

    /// Run evolution.
    fn run(&self, generations: usize) -> PyResult<String> {
        // TODO: Implement actual evolution
        Ok(format!(
            "Running {} generations with population {}",
            generations, self.population_size
        ))
    }
}
