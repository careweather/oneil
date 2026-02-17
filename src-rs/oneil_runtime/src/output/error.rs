//! Error types for runtime output operations.

use std::path::PathBuf;

use indexmap::IndexMap;
use oneil_shared::error::{AsOneilError, OneilError};
use std::io::Error as IoError;

/// Aggregated errors keyed by model path.
///
/// Each entry is either a file-level error (e.g. parse failure) or a collection
/// of evaluation errors (imports, parameters, tests).
#[derive(Debug, Default)]
pub struct RuntimeErrors {
    /// Map from model path to errors for that model.
    errors: IndexMap<PathBuf, ModelError>,
}

impl RuntimeErrors {
    /// Creates a new empty runtime errors collection.
    #[must_use]
    pub fn new() -> Self {
        Self {
            errors: IndexMap::new(),
        }
    }

    /// Adds a model error for the given path.
    ///
    /// If the path already has an error, it is replaced.
    pub fn add_model_error(&mut self, path: PathBuf, error: ModelError) {
        self.errors.insert(path, error);
    }

    /// Merges another collection of runtime errors into this one.
    ///
    /// Entries from `other` replace any existing entries for the same path.
    pub fn extend(&mut self, other: Self) {
        for (path, error) in other.errors {
            self.add_model_error(path, error);
        }
    }
}

/// Errors for a single model: either file-level or evaluation-level.
#[derive(Debug)]
pub enum ModelError {
    /// The file could not be read or parsed. Contains the reported errors.
    FileError(Vec<OneilError>),
    /// The model was loaded; contains import, parameter, and test errors.
    EvalErrors {
        /// Model reference name → error for that reference.
        model_import_errors: Box<IndexMap<String, OneilError>>,
        /// Python import path → error for that import.
        python_import_errors: Box<IndexMap<PathBuf, OneilError>>,
        /// Parameter name → list of errors for that parameter.
        parameter_errors: Box<IndexMap<String, Vec<OneilError>>>,
        /// Errors from model tests.
        test_errors: Vec<OneilError>,
    },
}

/// Error type for source loading failures.
#[derive(Debug)]
pub struct SourceError {
    path: PathBuf,
    error: IoError,
}

impl SourceError {
    /// Creates a new source error from a path and I/O error.
    #[must_use]
    pub const fn new(path: PathBuf, error: IoError) -> Self {
        Self { path, error }
    }
}

impl AsOneilError for SourceError {
    fn message(&self) -> String {
        format!("couldn't read `{}` - {}", self.path.display(), self.error)
    }
}
