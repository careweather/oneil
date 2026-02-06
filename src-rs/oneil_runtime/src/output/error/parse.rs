//! Error type for parse failures.

use oneil_shared::error::OneilError;

use crate::output::ast::Model;

use super::file::FileError;

/// Error type for parse failures.
///
/// Either a file error (e.g. source could not be read) or a list of parsing
/// errors together with a partial AST.
#[derive(Clone, Debug)]
pub enum ParseError {
    /// The source file could not be loaded.
    File(FileError),

    /// Parsing failed with one or more errors and an optional partial AST.
    ParseErrors {
        /// Errors produced by the parser.
        errors: Vec<OneilError>,
        /// The partial AST model, if any was produced before failure.
        partial_ast: Box<Model>,
    },
}

impl ParseError {
    /// Returns all underlying errors as a list of [`OneilError`]s.
    #[must_use]
    pub fn to_vec(&self) -> Vec<OneilError> {
        match self {
            Self::File(f) => f.to_vec(),
            Self::ParseErrors { errors, .. } => errors.clone(),
        }
    }
}
