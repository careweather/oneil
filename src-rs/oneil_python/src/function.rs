use pyo3::prelude::*;
use pyo3::types::PyTuple;

#[derive(Debug)]
pub struct PythonFunction {
    function: Py<PyAny>,
}

impl PythonFunction {
    pub const fn new(function: Py<PyAny>) -> Self {
        Self { function }
    }

    /// Calls the Python function with the given positional arguments.
    pub fn call<'py>(
        &self,
        py: Python<'py>,
        args: &[Bound<'py, PyAny>],
    ) -> PyResult<Bound<'py, PyAny>> {
        let callable = self.function.bind(py);
        let args_tuple = PyTuple::new(py, args)?;
        callable.call1(args_tuple)
    }
}
