use pyo3::prelude::*;

#[derive(Debug)]
pub struct PythonFunction {
    function: Py<PyAny>,
}

impl PythonFunction {
    pub const fn new(function: Py<PyAny>) -> Self {
        Self { function }
    }
}
