//! Error types for runtime output operations.

use std::path::PathBuf;

use indexmap::{IndexMap, IndexSet};

#[derive(Debug)]
pub struct TreeErrors {
    errors: IndexMap<PathBuf, TreeModelError>,
}

impl TreeErrors {
    pub fn empty() -> Self {
        Self {
            errors: IndexMap::new(),
        }
    }

    pub fn insert_model_error(&mut self, model_path: PathBuf) {
        self.errors.insert(model_path, TreeModelError::ModelError);
    }

    pub fn insert_parameter_error(&mut self, model_path: PathBuf, parameter_name: String) {
        if let Some(model_errors) = self.errors.get_mut(&model_path) {
            model_errors.insert_parameter_error(parameter_name);
        } else {
            let parameters = IndexSet::from_iter([parameter_name]);
            self.errors
                .insert(model_path, TreeModelError::ParamErrors { parameters });
        }
    }

    pub fn extend(&mut self, other: Self) {
        for (path, error) in other.errors {
            if let Some(model_errors) = self.errors.get_mut(&path) {
                model_errors.extend(error);
            } else {
                self.errors.insert(path, error);
            }
        }
    }
}

#[derive(Debug)]
pub enum TreeModelError {
    ModelError,
    ParamErrors { parameters: IndexSet<String> },
}

impl TreeModelError {
    pub fn insert_parameter_error(&mut self, parameter_name: String) {
        match self {
            Self::ModelError => (),
            Self::ParamErrors { parameters } => {
                parameters.insert(parameter_name);
            }
        }
    }

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

#[derive(Debug)]
pub enum GetValueError {
    Model,
    Parameter,
}
