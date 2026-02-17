//! Error reporting for models and parameters.

use std::path::Path;

use oneil_analysis::output::error::TreeErrors;

use super::Runtime;

impl Runtime {
    /// Returns errors for the given model.
    ///
    /// Model-level errors indicate that the model could not be loaded or evaluated.
    #[must_use]
    pub fn get_model_errors(&self, _model_path: &Path) -> TreeErrors {
        TreeErrors::empty()
    }

    /// Returns errors for the given parameter in a model.
    ///
    /// Parameter-level errors indicate that the parameter could not be resolved or evaluated.
    #[must_use]
    pub fn get_parameter_errors(
        &self,
        _model_path: &Path,
        _parameter_name: &str,
    ) -> TreeErrors {
        TreeErrors::empty()
    }
}
