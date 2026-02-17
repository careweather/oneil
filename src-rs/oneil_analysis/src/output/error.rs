//! Error types for runtime output operations.

use std::path::PathBuf;

use indexmap::{IndexMap, IndexSet};

/// Singleton error indicating that model evaluation had errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelEvalHasErrors;

/// Accumulated errors encountered while building a dependency or reference tree.
#[derive(Debug)]
pub struct TreeErrors {
    errors: IndexMap<PathBuf, TreeModelError>,
}

impl TreeErrors {
    /// Creates an empty error collection.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            errors: IndexMap::new(),
        }
    }

    /// Records a model-level error for the given path.
    pub fn insert_model_error(&mut self, model_path: PathBuf) {
        self.errors.insert(model_path, TreeModelError::ModelError);
    }

    /// Records a parameter-level error for the given model path and parameter name.
    pub fn insert_parameter_error(&mut self, model_path: PathBuf, parameter_name: String) {
        if let Some(model_errors) = self.errors.get_mut(&model_path) {
            model_errors.insert_parameter_error(parameter_name);
        } else {
            let parameters = IndexSet::from_iter([parameter_name]);
            self.errors
                .insert(model_path, TreeModelError::ParamErrors { parameters });
        }
    }

    /// Merges another `TreeErrors` into this one, combining errors per model path.
    pub fn extend(&mut self, other: Self) {
        for (path, error) in other.errors {
            if let Some(model_errors) = self.errors.get_mut(&path) {
                model_errors.extend(error);
            } else {
                self.errors.insert(path, error);
            }
        }
    }

    /// Returns an iterator over the model paths that have errors.
    pub fn model_paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.errors.keys()
    }
}

/// Errors for a single model when building a tree.
#[derive(Debug)]
pub enum TreeModelError {
    /// The model could not be loaded or evaluated.
    ModelError,
    /// The model loaded but some parameters had errors.
    ParamErrors {
        /// Names of parameters that had errors.
        parameters: IndexSet<String>,
    },
}

impl TreeModelError {
    /// Adds a parameter error to this model error.
    pub fn insert_parameter_error(&mut self, parameter_name: String) {
        match self {
            Self::ModelError => (),
            Self::ParamErrors { parameters } => {
                parameters.insert(parameter_name);
            }
        }
    }

    /// Merges another `TreeModelError` into this one.
    pub fn extend(&mut self, other: Self) {
        match (self, other) {
            (Self::ModelError, _) => (),
            (self_, other @ Self::ModelError) => *self_ = other,
            (
                Self::ParamErrors { parameters },
                Self::ParamErrors {
                    parameters: other_parameters,
                },
            ) => {
                parameters.extend(other_parameters);
            }
        }
    }
}

/// Set of model paths that had errors during independents analysis.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IndependentsErrors {
    /// Model paths that could not be evaluated or had evaluation errors.
    paths: IndexSet<PathBuf>,
}

impl IndependentsErrors {
    /// Creates an empty error set.
    #[must_use]
    pub fn new() -> Self {
        Self {
            paths: IndexSet::new(),
        }
    }

    /// Adds a model path that had an error.
    pub fn insert(&mut self, path: PathBuf) {
        self.paths.insert(path);
    }

    /// Returns whether no errors were recorded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }

    /// Returns an iterator over the model paths that had errors.
    #[must_use]
    pub fn paths(&self) -> indexmap::set::Iter<'_, PathBuf> {
        self.paths.iter()
    }

    /// Adds all paths from another `IndependentsErrors` into this one.
    pub fn extend(&mut self, other: Self) {
        self.paths.extend(other.paths);
    }
}

/// Error when looking up a value for a tree node.
#[derive(Debug)]
pub enum GetValueError {
    /// The model was not found or could not be evaluated.
    Model,
    /// The parameter was not found in the model.
    Parameter,
}
