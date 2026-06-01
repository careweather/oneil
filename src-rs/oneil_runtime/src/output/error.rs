//! Error types for runtime output operations.

use std::path::PathBuf;

use indexmap::IndexMap;
use oneil_shared::{
    error::OneilDiagnostic,
    paths::{ModelPath, PythonPath},
    symbols::{ParameterName, ReferenceName, TestIndex},
};

/// Aggregated errors keyed by model path.
///
/// Each entry is either a file-level error (e.g. parse failure) or a collection
/// of evaluation errors (imports, parameters, tests).
#[derive(Debug, Default)]
pub struct RuntimeErrors {
    /// Map from model path to errors for that model.
    errors: Box<IndexMap<ModelPath, ModelError>>,

    /// Map from Python import path to errors for that import.
    python_import_errors: Box<IndexMap<PythonPath, OneilDiagnostic>>,

    /// Map from Python call cache file path to warnings for that file.
    cache_warnings: Box<IndexMap<PathBuf, Vec<OneilDiagnostic>>>,
}

impl RuntimeErrors {
    /// Creates a new empty runtime errors collection.
    #[must_use]
    pub fn new() -> Self {
        Self {
            errors: Box::new(IndexMap::new()),
            python_import_errors: Box::new(IndexMap::new()),
            cache_warnings: Box::new(IndexMap::new()),
        }
    }

    /// Adds a cache warning for the given cache file path.
    pub fn add_cache_warning(&mut self, path: PathBuf, diagnostic: OneilDiagnostic) {
        self.cache_warnings
            .entry(path)
            .or_default()
            .push(diagnostic);
    }

    /// Merges cache warnings from an iterator of diagnostics.
    ///
    /// Each diagnostic's path is used as the cache file key.
    pub fn extend_cache_warnings(
        &mut self,
        diagnostics: impl IntoIterator<Item = OneilDiagnostic>,
    ) {
        for diagnostic in diagnostics {
            self.add_cache_warning(diagnostic.path().clone(), diagnostic);
        }
    }

    /// Adds a model error for the given path.
    ///
    /// If the path already has an error, it is replaced.
    pub fn add_model_error(&mut self, path: ModelPath, error: ModelError) {
        self.errors.insert(path, error);
    }

    /// Adds a Python import error for the given path.
    ///
    /// If the path already has an error, it is replaced.
    pub fn add_python_import_error(&mut self, path: PythonPath, error: OneilDiagnostic) {
        self.python_import_errors.insert(path, error);
    }

    /// Merges another collection of runtime errors into this one.
    ///
    /// Entries from `other` replace any existing entries for the same path.
    pub fn extend(&mut self, other: Self) {
        for (path, error) in *other.errors {
            self.add_model_error(path, error);
        }

        for (path, error) in *other.python_import_errors {
            self.add_python_import_error(path, error);
        }

        for (path, warnings) in *other.cache_warnings {
            for warning in warnings {
                self.add_cache_warning(path.clone(), warning);
            }
        }
    }

    /// Converts the errors to a vector of Oneil errors.
    #[must_use]
    pub fn to_vec(&self) -> Vec<&OneilDiagnostic> {
        self.errors
            .values()
            .flat_map(|error| match error {
                ModelError::FileError(errors) => errors.iter().collect::<Vec<&OneilDiagnostic>>(),
                ModelError::EvalErrors {
                    model_import_errors,
                    python_import_errors,
                    parameter_errors,
                    test_errors,
                    design_resolution_errors,
                } => model_import_errors
                    .values()
                    .chain(python_import_errors.values())
                    .chain(parameter_errors.values().flatten())
                    .chain(test_errors.values().flatten())
                    .chain(design_resolution_errors.iter())
                    .collect(),
            })
            .chain(self.python_import_errors.values())
            .chain(self.cache_warnings.values().flatten())
            .collect()
    }

    /// Converts the errors to a map of model paths to errors.
    #[must_use]
    pub fn to_map(&self) -> IndexMap<PathBuf, Vec<OneilDiagnostic>> {
        self.errors
            .iter()
            .map(|(path, error)| {
                (
                    path.clone().into_path_buf(),
                    error.get_all_errors().into_iter().cloned().collect(),
                )
            })
            .chain(
                self.python_import_errors
                    .iter()
                    .map(|(path, error)| (path.clone().into_path_buf(), vec![error.clone()])),
            )
            .chain(
                self.cache_warnings
                    .iter()
                    .map(|(path, warnings)| (path.clone(), warnings.clone())),
            )
            .collect()
    }

    /// Returns true if there are no errors.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Errors for a single model: either file-level or evaluation-level.
#[derive(Debug)]
pub enum ModelError {
    /// The file could not be read or parsed. Contains the reported errors.
    FileError(Vec<OneilDiagnostic>),
    /// The model was loaded; contains import, parameter, and test errors.
    EvalErrors {
        /// Model reference name → error for that reference.
        model_import_errors: Box<IndexMap<ReferenceName, OneilDiagnostic>>,
        /// Python import path → error for that import.
        python_import_errors: Box<IndexMap<PythonPath, OneilDiagnostic>>,
        /// Parameter name → list of errors for that parameter.
        parameter_errors: Box<IndexMap<ParameterName, Vec<OneilDiagnostic>>>,
        /// Errors from model tests.
        test_errors: Box<IndexMap<TestIndex, Vec<OneilDiagnostic>>>,
        /// Design / `apply` resolution errors not tied to a single parameter.
        design_resolution_errors: Box<Vec<OneilDiagnostic>>,
    },
}

impl ModelError {
    /// Returns all Oneil errors in this model error as a vector of references.
    #[must_use]
    pub fn get_all_errors(&self) -> Vec<&OneilDiagnostic> {
        match self {
            Self::FileError(errors) => errors.iter().collect(),
            Self::EvalErrors {
                model_import_errors,
                python_import_errors,
                parameter_errors,
                test_errors,
                design_resolution_errors,
            } => model_import_errors
                .values()
                .chain(python_import_errors.values())
                .chain(parameter_errors.values().flatten())
                .chain(test_errors.values().flatten())
                .chain(design_resolution_errors.iter())
                .collect(),
        }
    }
}
