//! Error types for Python integration.

use oneil_shared::error::AsOneilError;

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
