//! Error type for dependency tree operations.

use indexmap::IndexMap;
use oneil_shared::error::OneilError;

use crate::output::error::ParseError;

/// Error type for tree operations, parameterized by the tree value type.
///
/// Either a resolution error, or a partial tree together with a map of
/// parameter names to their resolution/evaluation errors.
#[derive(Debug, Clone)]
pub enum TreeError {
    /// Parse error when building or resolving the tree.
    Parse(ParseError),
    /// Map of parameter names to their resolution/evaluation errors.
    TreeErrors {
        /// Errors keyed by parameter name.
        parameter_errors: IndexMap<String, Vec<OneilError>>,
    },
}

impl TreeError {
    /// Inserts errors for a single parameter. No-op if `self` is
    /// [`Parse`](Self::Parse). If `self` is [`TreeErrors`](Self::TreeErrors)
    /// and the parameter is not yet present, inserts `errors`; otherwise leaves
    /// existing errors for that parameter unchanged.
    pub fn insert_parameter_errors(&mut self, name: String, errors: Vec<OneilError>) {
        if let Self::TreeErrors { parameter_errors } = self {
            parameter_errors.entry(name).or_insert(errors);
        }
    }

    /// Inserts all errors from `other` into `self`. If `other` is a
    /// [`Parse`](Self::Parse), `self` is replaced with it. If both are
    /// [`TreeErrors`](Self::TreeErrors), entries from `other` are merged
    /// in; for parameters already present in `self`, the existing errors are
    /// kept and `other`'s errors are not added.
    pub fn insert_all(&mut self, other: Self) {
        match (self, other) {
            (self_ref, other @ Self::Parse(_)) => *self_ref = other,
            (Self::Parse(_), _) => {}
            (
                Self::TreeErrors { parameter_errors },
                Self::TreeErrors {
                    parameter_errors: other_errors,
                },
            ) => {
                for (k, v) in other_errors {
                    parameter_errors.entry(k).or_insert(v);
                }
            }
        }
    }

    /// Returns all underlying errors as a list of [`OneilError`]s.
    #[must_use]
    pub fn to_vec(&self) -> Vec<OneilError> {
        match self {
            Self::Parse(p) => p.to_vec(),
            Self::TreeErrors { parameter_errors } => {
                parameter_errors.values().flatten().cloned().collect()
            }
        }
    }
}

impl Default for TreeError {
    fn default() -> Self {
        Self::TreeErrors {
            parameter_errors: IndexMap::new(),
        }
    }
}
