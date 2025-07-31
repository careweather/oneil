use oneil_error::{AsOneilError, AsOneilErrorWithSource};
use oneil_ir::{
    reference::{Identifier, ModelPath},
    span::Span,
};

/// Represents an error that occurred during submodel resolution.
///
/// This error type is used when a `use model` declaration cannot be resolved to
/// its corresponding model. This can happen when the referenced model has errors
/// or when the submodel identifier is not defined in the referenced model.
#[derive(Debug, Clone, PartialEq)]
pub enum SubmodelResolutionError {
    /// The referenced model has errors, preventing submodel resolution.
    ModelHasError {
        /// The path of the model that has errors
        model_path: ModelPath,
        /// The span of where the model is referenced
        reference_span: Span,
    },
    /// The submodel identifier is not defined in the referenced model.
    UndefinedSubmodel {
        /// The path of the model that contains the submodel
        parent_model_path: ModelPath,
        /// The identifier of the submodel that is undefined
        submodel: Identifier,
        /// The span of where the submodel is referenced
        reference_span: Span,
    },
}

impl SubmodelResolutionError {
    /// Creates a new error indicating that the referenced model has errors.
    ///
    /// # Arguments
    ///
    /// * `model_path` - The path of the model that has errors
    /// * `reference_span` - The span of where the model is referenced
    ///
    /// # Returns
    ///
    /// A new `SubmodelResolutionError::ModelHasError` variant.
    pub fn model_has_error(model_path: ModelPath, reference_span: Span) -> Self {
        Self::ModelHasError {
            model_path,
            reference_span,
        }
    }

    /// Creates a new error indicating that the submodel is undefined in the referenced model.
    ///
    /// # Arguments
    ///
    /// * `parent_model_path` - The path of the parent model that contains the submodel reference
    /// * `identifier` - The identifier of the undefined submodel
    /// * `reference_span` - The span of where the submodel is referenced
    ///
    /// # Returns
    ///
    /// A new `SubmodelResolutionError::UndefinedSubmodel` variant.
    pub fn undefined_submodel_in_submodel(
        parent_model_path: ModelPath,
        submodel: Identifier,
        reference_span: Span,
    ) -> Self {
        Self::UndefinedSubmodel {
            parent_model_path,
            submodel,
            reference_span,
        }
    }

    /// Converts the submodel resolution error to a string representation.
    ///
    /// This method delegates to the display module to format the error message
    /// in a user-friendly way.
    ///
    /// # Returns
    ///
    /// A string representation of the submodel resolution error.
    pub fn to_string(&self) -> String {
        match self {
            SubmodelResolutionError::ModelHasError {
                model_path: path,
                reference_span: _,
            } => {
                let path = path.as_ref().display();
                format!("submodel `{}` has errors", path)
            }
            SubmodelResolutionError::UndefinedSubmodel {
                parent_model_path,
                submodel,
                reference_span: _,
            } => {
                let path = parent_model_path.as_ref().display();
                format!(
                    "submodel `{}` is not defined in model `{}`",
                    submodel.as_str(),
                    path
                )
            }
        }
    }
}

impl AsOneilError for SubmodelResolutionError {
    fn message(&self) -> String {
        self.to_string()
    }
}

impl AsOneilErrorWithSource for SubmodelResolutionError {
    fn error_location(&self, source: &str) -> oneil_error::ErrorLocation {
        match self {
            SubmodelResolutionError::ModelHasError {
                model_path: _,
                reference_span,
            } => {
                let start = reference_span.start();
                let length = reference_span.length();
                oneil_error::ErrorLocation::from_source_and_span(source, start, length)
            }
            SubmodelResolutionError::UndefinedSubmodel {
                parent_model_path: _,
                submodel: _,
                reference_span,
            } => {
                let start = reference_span.start();
                let length = reference_span.length();
                oneil_error::ErrorLocation::from_source_and_span(source, start, length)
            }
        }
    }
}
