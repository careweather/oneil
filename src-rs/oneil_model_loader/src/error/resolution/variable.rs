use oneil_ir::{
    reference::{Identifier, ModelPath},
    span::WithSpan,
};

/// Represents an error that occurred during variable resolution within expressions.
///
/// This error type is used when a variable reference within an expression cannot
/// be resolved. This is the most fundamental resolution error type, as it represents
/// the failure to resolve a simple identifier to its definition.
#[derive(Debug, Clone, PartialEq)]
pub enum VariableResolutionError {
    /// The model that should contain the variable has errors.
    ModelHasError(ModelPath),
    /// The parameter that should contain the variable has errors.
    ParameterHasError(Identifier),
    /// The resolution of a submodel that is referenced by a variable has failed.
    SubmodelResolutionFailed(Identifier),
    /// The parameter is not defined in the current context.
    ///
    /// Note that the span associated with the identifier is the span of the identifier
    /// in the expression that references the parameter.
    UndefinedParameter(Option<ModelPath>, WithSpan<Identifier>),
    /// The submodel is not defined in the current context.
    ///
    /// Note that the span associated with the identifier is the span of the identifier
    /// in the expression that references the submodel.
    UndefinedSubmodel(Option<ModelPath>, WithSpan<Identifier>),
}

impl VariableResolutionError {
    /// Creates a new error indicating that the model has errors.
    ///
    /// # Arguments
    ///
    /// * `model_path` - The path of the model that has errors
    ///
    /// # Returns
    ///
    /// A new `VariableResolutionError::ModelHasError` variant.
    pub fn model_has_error(model_path: ModelPath) -> Self {
        Self::ModelHasError(model_path)
    }

    /// Creates a new error indicating that the parameter has errors.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The identifier of the parameter that has errors
    ///
    /// # Returns
    ///
    /// A new `VariableResolutionError::ParameterHasError` variant.
    pub fn parameter_has_error(identifier: Identifier) -> Self {
        Self::ParameterHasError(identifier)
    }

    /// Creates a new error indicating that resolution of a submodel that is
    /// referenced by a variable has failed.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The identifier of the submodel that has errors
    ///
    /// # Returns
    ///
    /// A new `VariableResolutionError::SubmodelResolutionFailed` variant.
    pub fn submodel_resolution_failed(identifier: Identifier) -> Self {
        Self::SubmodelResolutionFailed(identifier)
    }

    /// Creates a new error indicating that the parameter is undefined.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The identifier of the undefined parameter
    ///
    /// # Returns
    ///
    /// A new `VariableResolutionError::UndefinedParameter` variant.
    pub fn undefined_parameter(identifier: WithSpan<Identifier>) -> Self {
        Self::UndefinedParameter(None, identifier)
    }

    /// Creates a new error indicating that the parameter is undefined in a specific submodel.
    ///
    /// # Arguments
    ///
    /// * `submodel_path` - The path of the submodel where the parameter should be defined
    /// * `identifier` - The identifier of the undefined parameter
    ///
    /// # Returns
    ///
    /// A new `VariableResolutionError::UndefinedParameter` variant.
    pub fn undefined_parameter_in_submodel(
        submodel_path: ModelPath,
        identifier: WithSpan<Identifier>,
    ) -> Self {
        Self::UndefinedParameter(Some(submodel_path), identifier)
    }

    /// Creates a new error indicating that the submodel is undefined.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The identifier of the undefined submodel
    ///
    /// # Returns
    ///
    /// A new `VariableResolutionError::UndefinedSubmodel` variant.
    pub fn undefined_submodel(identifier: WithSpan<Identifier>) -> Self {
        Self::UndefinedSubmodel(None, identifier)
    }

    /// Creates a new error indicating that the submodel is undefined in a specific parent model.
    ///
    /// # Arguments
    ///
    /// * `parent_model_path` - The path of the parent model where the submodel should be defined
    /// * `identifier` - The identifier of the undefined submodel
    ///
    /// # Returns
    ///
    /// A new `VariableResolutionError::UndefinedSubmodel` variant.
    pub fn undefined_submodel_in_submodel(
        parent_model_path: ModelPath,
        identifier: WithSpan<Identifier>,
    ) -> Self {
        Self::UndefinedSubmodel(Some(parent_model_path), identifier)
    }

    /// Converts the variable resolution error to a string representation.
    ///
    /// This method delegates to the display module to format the error message
    /// in a user-friendly way.
    ///
    /// # Returns
    ///
    /// A string representation of the variable resolution error.
    pub fn to_string(&self) -> String {
        match self {
            VariableResolutionError::ModelHasError(model_path) => {
                let path = model_path.as_ref().display();
                format!("model `{}` has errors", path)
            }
            VariableResolutionError::ParameterHasError(identifier) => {
                let identifier = identifier.as_str();
                format!("parameter `{}` has errors", identifier)
            }
            VariableResolutionError::SubmodelResolutionFailed(identifier) => {
                let identifier = identifier.as_str();
                format!("submodel `{}` resolution failed", identifier)
            }
            VariableResolutionError::UndefinedParameter(model_path, identifier) => {
                let identifier_str = identifier.value().as_str();
                match model_path {
                    Some(path) => format!(
                        "parameter `{}` is not defined in model `{}`",
                        identifier_str,
                        path.as_ref().display()
                    ),
                    None => format!(
                        "parameter `{}` is not defined in the current model",
                        identifier_str
                    ),
                }
            }
            VariableResolutionError::UndefinedSubmodel(model_path, identifier) => {
                let identifier_str = identifier.value().as_str();
                match model_path {
                    Some(path) => format!(
                        "submodel `{}` is not defined in model `{}`",
                        identifier_str,
                        path.as_ref().display()
                    ),
                    None => format!(
                        "submodel `{}` is not defined in the current model",
                        identifier_str
                    ),
                }
            }
        }
    }
}
