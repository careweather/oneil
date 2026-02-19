//! Error types for Python integration.

use oneil_shared::error::{AsOneilError, Context};
use oneil_shared::span::Span;
use pyo3::Python;
use pyo3::types::PyTracebackMethods;

/// Error that can occur when loading a Python import.
#[derive(Debug)]
pub enum LoadPythonImportError {
    /// The source string contains a null byte, which cannot be passed to Python.
    SourceHasNullByte,
    /// Python raised an error while loading the module.
    CouldNotLoadPythonModule(pyo3::PyErr),
}

impl AsOneilError for LoadPythonImportError {
    fn message(&self) -> String {
        match self {
            Self::SourceHasNullByte => "Python source contains a null byte".to_string(),
            Self::CouldNotLoadPythonModule(_error) => "Could not load Python module".to_string(),
        }
    }

    fn context(&self) -> Vec<Context> {
        match self {
            Self::SourceHasNullByte => vec![],
            Self::CouldNotLoadPythonModule(error) => Python::attach(|py| {
                let traceback = error.traceback(py).and_then(|tb| tb.format().ok());

                traceback.map_or_else(
                    || vec![Context::Note(error.to_string())],
                    |traceback| vec![Context::Note(error.to_string()), Context::Note(traceback)],
                )
            }),
        }
    }
}

/// Error from evaluating a Python function (argument conversion, call, or result conversion failed).
#[derive(Debug)]
pub struct PythonEvalError {
    /// Name of the Python function that was called.
    pub function_name: String,
    /// Span of the function identifier in the Oneil source.
    pub identifier_span: Span,
    /// Error message from Python or from conversion.
    pub message: String,
    /// The traceback from Python.
    pub traceback: Option<String>,
}
