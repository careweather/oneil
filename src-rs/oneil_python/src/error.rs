//! Error types for Python integration.

use oneil_shared::error::AsOneilError;
use oneil_shared::span::Span;

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
            Self::CouldNotLoadPythonModule(error) => {
                format!("Could not load Python module: {error}")
            }
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
}
