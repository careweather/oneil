//! Error type for model evaluation failures.

use indexmap::IndexMap;
use oneil_eval::output;
use oneil_shared::error::OneilError;

use super::resolution::ResolutionError;

/// Error type for model evaluation failures.
///
/// Either a resolution error (e.g. IR could not be loaded) or a partial
/// evaluation result together with parameter and test errors for a single model.
#[derive(Clone, Debug)]
pub enum EvalError {
    /// Evaluation failed due to a resolution error (e.g. parse or resolution failure).
    Resolution(ResolutionError),

    /// Evaluation produced a partial result and parameter/test errors for one model.
    EvalErrors {
        /// The partial evaluation result for the model.
        partial_result: output::Model,
        /// Parameter errors (parameter name to list of errors).
        parameter_errors: IndexMap<String, Vec<OneilError>>,
        /// Test errors for the model.
        test_errors: Vec<OneilError>,
    },
}

impl EvalError {
    /// Returns all underlying errors as a list of [`OneilError`]s.
    #[must_use]
    pub fn to_vec(&self) -> Vec<OneilError> {
        match self {
            EvalError::Resolution(r) => r.to_vec(),
            EvalError::EvalErrors {
                parameter_errors,
                test_errors,
                ..
            } => {
                let mut v = test_errors.clone();
                v.extend(parameter_errors.values().flatten().cloned());
                v
            }
        }
    }
}
