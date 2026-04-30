use std::path::PathBuf;
use std::sync::Arc;

use indexmap::{IndexMap, IndexSet};
use oneil_shared::symbols::PyFunctionName;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

#[derive(Debug, Default, Clone)]
pub struct PythonModule {
    docs: Option<String>,
    functions: IndexMap<PyFunctionName, PythonFunction>,
    imports: IndexSet<PathBuf>,
    hash: u64,
}

impl PythonModule {
    pub const fn new(
        docs: Option<String>,
        functions: IndexMap<PyFunctionName, PythonFunction>,
        imports: IndexSet<PathBuf>,
        hash: u64,
    ) -> Self {
        Self {
            docs,
            functions,
            imports,
            hash,
        }
    }

    pub fn get_function(&self, identifier: &PyFunctionName) -> Option<&PythonFunction> {
        self.functions.get(identifier)
    }

    pub fn get_function_names(&self) -> impl Iterator<Item = &PyFunctionName> {
        self.functions.keys()
    }

    pub fn get_docs(&self) -> Option<&str> {
        self.docs.as_deref()
    }

    pub const fn get_imports(&self) -> &IndexSet<PathBuf> {
        &self.imports
    }

    pub const fn get_hash(&self) -> u64 {
        self.hash
    }
}

#[derive(Debug, Clone)]
pub struct PythonFunction {
    function: Arc<Py<PyAny>>,
    docs: Option<String>,
    line_no: Option<u32>,
}

impl PythonFunction {
    pub fn new(function: Py<PyAny>, docs: Option<String>, line_no: Option<u32>) -> Self {
        let function = Arc::new(function);
        Self {
            function,
            docs,
            line_no,
        }
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

    /// Returns the documentation string for the function.
    pub fn get_docs(&self) -> Option<&str> {
        self.docs.as_deref()
    }

    /// Returns the line number of the function.
    pub const fn get_line_no(&self) -> Option<u32> {
        self.line_no
    }
}
