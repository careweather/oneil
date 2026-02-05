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
#[derive(Clone, Debug)]
pub enum ResolutionError {
    /// Resolution failed due to a parse error (source or AST loading).
    Parse(ParseError),

    /// Resolution produced a (possibly partial) IR model and resolution errors.
    ResolutionErrors {
        /// The IR model (possibly partial).
        partial_ir: ir::Model,
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
