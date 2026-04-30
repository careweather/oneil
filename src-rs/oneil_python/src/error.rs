//! Error types for Python integration.

use std::io;
use std::path::PathBuf;

use indexmap::IndexMap;
use oneil_shared::error::{AsOneilDiagnostic, Context, DiagnosticKind};
use pyo3::Python;
use pyo3::types::PyTracebackMethods;
use serde::{Deserialize, Serialize};

/// Error that can occur when loading a Python import.
#[derive(Debug)]
pub enum LoadPythonImportError {
    /// The source string contains a null byte, which cannot be passed to Python.
    SourceHasNullByte,
    /// Python raised an error while loading the module.
    CouldNotLoadPythonModule(pyo3::PyErr),
    /// Could not calculate the source hash.
    CouldNotCalculateSourceHash {
        /// File errors.
        file_errors: Box<IndexMap<PathBuf, io::Error>>,
    },
}

impl AsOneilDiagnostic for LoadPythonImportError {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn message(&self) -> String {
        match self {
            Self::SourceHasNullByte => "Python source contains a null byte".to_string(),
            Self::CouldNotLoadPythonModule(_error) => "could not load Python module".to_string(),
            Self::CouldNotCalculateSourceHash { file_errors: _ } => {
                "Could not calculate source hash".to_string()
            }
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
            Self::CouldNotCalculateSourceHash { file_errors } => file_errors
                .iter()
                .map(|(path, error)| {
                    Context::Note(format!("could not read {}: {}", path.display(), error))
                })
                .collect(),
        }
    }
}

/// Error from evaluating a Python function (argument conversion, call, or result conversion failed).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "error")]
#[serde(rename_all = "snake_case")]
pub enum PythonEvalError {
    PyErr {
        /// Error message from Python or from conversion.
        message: String,
        /// The traceback from Python.
        traceback: Option<String>,
    },

    InvalidReturnValue {
        /// The value that was invalid.
        value_repr: String,
    },
}
