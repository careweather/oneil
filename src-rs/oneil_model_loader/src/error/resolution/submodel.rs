use oneil_ir::{
    reference::{Identifier, ModelPath},
    span::WithSpan,
};

/// Represents an error that occurred during submodel resolution.
///
/// This error type is used when a `use model` declaration cannot be resolved to
/// its corresponding model. This can happen when the referenced model has errors
/// or when the submodel identifier is not defined in the referenced model.
#[derive(Debug, Clone, PartialEq)]
pub enum SubmodelResolutionError {
    /// The referenced model has errors, preventing submodel resolution.
    ModelHasError(ModelPath),
    /// The submodel identifier is not defined in the referenced model.
    UndefinedSubmodel(Option<ModelPath>, WithSpan<Identifier>),
}

impl SubmodelResolutionError {
    /// Creates a new error indicating that the referenced model has errors.
    ///
    /// # Arguments
    ///
    /// * `model_path` - The path of the model that has errors
    ///
    /// # Returns
    ///
    /// A new `SubmodelResolutionError::ModelHasError` variant.
    pub fn model_has_error(model_path: ModelPath) -> Self {
        Self::ModelHasError(model_path)
    }

    /// Creates a new error indicating that the submodel is undefined in the referenced model.
    ///
    /// # Arguments
    ///
    /// * `parent_model_path` - The path of the parent model that contains the submodel reference
    /// * `identifier` - The identifier of the undefined submodel
    ///
    /// # Returns
    ///
    /// A new `SubmodelResolutionError::UndefinedSubmodel` variant.
    pub fn undefined_submodel_in_submodel(
        parent_model_path: ModelPath,
        identifier: WithSpan<Identifier>,
    ) -> Self {
        Self::UndefinedSubmodel(Some(parent_model_path), identifier)
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
            SubmodelResolutionError::ModelHasError(model_path) => {
                let path = model_path.as_ref().display();
                format!("submodel `{}` has errors", path)
            }
            SubmodelResolutionError::UndefinedSubmodel(parent_model_path, identifier) => {
                let identifier = identifier.value();
                match parent_model_path {
                    Some(path) => {
                        let path = path.as_ref().display();
                        format!(
                            "submodel `{}` is not defined in model `{}`",
                            identifier.as_str(),
                            path
                        )
                    }
                    None => format!("submodel `{}` is not defined", identifier.as_str()),
                }
            }
        }
    }
}
