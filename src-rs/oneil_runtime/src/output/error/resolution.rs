//! Error type for model resolution failures.

use std::path::PathBuf;

use indexmap::IndexMap;
use oneil_ir as ir;
use oneil_shared::error::OneilError;

use super::parse::ParseError;

/// Error type for model resolution failures.
///
/// Either a parse error (e.g. source or AST could not be loaded) or a partial
/// IR model together with circular dependency, python import, model import,
/// parameter, and test resolution errors (as [`OneilError`]s).
#[expect(
    clippy::large_enum_variant,
    reason = "aside from creating a whole new struct for `ResolutionErrors`, there is no other way to represent this error"
)]
#[derive(Clone, Debug)]
pub enum ResolutionError {
    /// Resolution failed due to a parse error (source or AST loading).
    Parse(ParseError),

    /// Resolution produced a (possibly partial) IR model and resolution errors.
    ResolutionErrors {
        /// The IR model (possibly partial).
        partial_ir: Box<ir::Model>,
        /// Circular dependency errors.
        circular_dependency_errors: Vec<OneilError>,
        /// Python import resolution errors.
        python_import_errors: IndexMap<PathBuf, Vec<OneilError>>,
        /// Model import resolution errors.
        model_import_errors: IndexMap<PathBuf, Vec<OneilError>>,
        /// Parameter resolution errors.
        parameter_errors: IndexMap<String, Vec<OneilError>>,
        /// Test resolution errors.
        test_errors: Vec<OneilError>,
    },
}

impl ResolutionError {
    /// Returns all underlying errors as a list of [`OneilError`]s.
    #[must_use]
    pub fn to_vec(&self) -> Vec<OneilError> {
        match self {
            Self::Parse(p) => p.to_vec(),
            Self::ResolutionErrors {
                circular_dependency_errors,
                python_import_errors,
                model_import_errors,
                parameter_errors,
                test_errors,
                ..
            } => {
                let mut v = circular_dependency_errors.clone();
                v.extend(python_import_errors.values().flatten().cloned());
                v.extend(model_import_errors.values().flatten().cloned());
                v.extend(parameter_errors.values().flatten().cloned());
                v.extend(test_errors.iter().cloned());
                v
            }
        }
    }
}
