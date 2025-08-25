use std::fmt;

use oneil_error::{AsOneilError, ErrorLocation};
use oneil_ir::{
    reference::{Identifier, ModelPath},
    span::Span,
};

/// Represents an error that occurred during variable resolution within expressions.
///
/// This error type is used when a variable reference within an expression cannot
/// be resolved. This is the most fundamental resolution error type, as it represents
/// the failure to resolve a simple identifier to its definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariableResolutionError {
    /// The model that should contain the variable has errors.
    ModelHasError {
        /// The path of the model that has errors
        path: ModelPath,
        /// The span of where the model is referenced
        reference_span: Span,
    },
    /// The parameter that should contain the variable has errors.
    ParameterHasError {
        /// The identifier of the parameter that has errors
        identifier: Identifier,
        /// The span of where the parameter is referenced
        reference_span: Span,
    },
    /// The resolution of a submodel that is referenced by a variable has failed.
    SubmodelResolutionFailed {
        /// The identifier of the submodel that has errors
        identifier: Identifier,
        /// The span of where the submodel is referenced
        reference_span: Span,
    },
    /// The parameter is not defined in the current context.
    UndefinedParameter {
        /// The path of the model that contains the parameter (if None, the parameter is not defined in the current model)
        model_path: Option<ModelPath>,
        /// The identifier of the parameter that is undefined
        parameter: Identifier,
        /// The span of where the parameter is referenced
        reference_span: Span,
    },
    /// The submodel is not defined in the current context.
    UndefinedSubmodel {
        /// The path of the model that contains the submodel (if None, the submodel is not defined in the current model)
        model_path: Option<ModelPath>,
        /// The identifier of the submodel that is undefined
        submodel: Identifier,
        /// The span of where the submodel is referenced
        reference_span: Span,
    },
}

impl VariableResolutionError {
    /// Creates a new error indicating that the model has errors.
    ///
    /// # Arguments
    ///
    /// * `model_path` - The path of the model that has errors
    /// * `reference_span` - The span of where the model is referenced
    ///
    /// # Returns
    ///
    /// A new `VariableResolutionError::ModelHasError` variant.
    #[must_use]
    pub const fn model_has_error(model_path: ModelPath, reference_span: Span) -> Self {
        Self::ModelHasError {
            path: model_path,
            reference_span,
        }
    }

    /// Creates a new error indicating that the parameter has errors.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The identifier of the parameter that has errors
    /// * `reference_span` - The span of where the parameter is referenced
    ///
    /// # Returns
    ///
    /// A new `VariableResolutionError::ParameterHasError` variant.
    #[must_use]
    pub const fn parameter_has_error(identifier: Identifier, reference_span: Span) -> Self {
        Self::ParameterHasError {
            identifier,
            reference_span,
        }
    }

    /// Creates a new error indicating that resolution of a submodel that is
    /// referenced by a variable has failed.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The identifier of the submodel that has errors
    /// * `reference_span` - The span of where the submodel is referenced
    ///
    /// # Returns
    ///
    /// A new `VariableResolutionError::SubmodelResolutionFailed` variant.
    #[must_use]
    pub const fn submodel_resolution_failed(identifier: Identifier, reference_span: Span) -> Self {
        Self::SubmodelResolutionFailed {
            identifier,
            reference_span,
        }
    }

    /// Creates a new error indicating that the parameter is undefined in the current model.
    ///
    /// # Arguments
    ///
    /// * `parameter` - The identifier of the undefined parameter
    /// * `reference_span` - The span of where the parameter is referenced
    ///
    /// # Returns
    ///
    /// A new `VariableResolutionError::UndefinedParameter` variant.
    #[must_use]
    pub const fn undefined_parameter(parameter: Identifier, reference_span: Span) -> Self {
        Self::UndefinedParameter {
            model_path: None,
            parameter,
            reference_span,
        }
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
    #[must_use]
    pub const fn undefined_parameter_in_submodel(
        submodel_path: ModelPath,
        parameter: Identifier,
        reference_span: Span,
    ) -> Self {
        Self::UndefinedParameter {
            model_path: Some(submodel_path),
            parameter,
            reference_span,
        }
    }

    /// Creates a new error indicating that the submodel is undefined in the current model.
    ///
    /// # Arguments
    ///
    /// * `submodel` - The identifier of the undefined submodel
    /// * `reference_span` - The span of where the submodel is referenced
    ///
    /// # Returns
    ///
    /// A new `VariableResolutionError::UndefinedSubmodel` variant.
    #[must_use]
    pub const fn undefined_submodel(submodel: Identifier, reference_span: Span) -> Self {
        Self::UndefinedSubmodel {
            model_path: None,
            submodel,
            reference_span,
        }
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
    #[must_use]
    pub const fn undefined_submodel_in_submodel(
        parent_model_path: ModelPath,
        submodel: Identifier,
        reference_span: Span,
    ) -> Self {
        Self::UndefinedSubmodel {
            model_path: Some(parent_model_path),
            submodel,
            reference_span,
        }
    }
}

impl fmt::Display for VariableResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ModelHasError {
                path,
                reference_span: _,
            } => {
                let path = path.as_ref().display();
                write!(f, "model `{path}` has errors")
            }
            Self::ParameterHasError {
                identifier,
                reference_span: _,
            } => {
                let identifier = identifier.as_str();
                write!(f, "parameter `{identifier}` has errors")
            }
            Self::SubmodelResolutionFailed {
                identifier,
                reference_span: _,
            } => {
                let identifier = identifier.as_str();
                write!(f, "unable to resolve submodel `{identifier}`")
            }
            Self::UndefinedParameter {
                model_path,
                parameter,
                reference_span: _,
            } => {
                // TODO: add context "did you mean `{}`?" using hamming distance to suggest similar parameter names
                let identifier_str = parameter.as_str();
                match model_path {
                    Some(path) => {
                        let path = path.as_ref().display();
                        write!(
                            f,
                            "parameter `{identifier_str}` is not defined in model `{path}`"
                        )
                    }
                    None => write!(
                        f,
                        "parameter `{identifier_str}` is not defined in the current model"
                    ),
                }
            }
            Self::UndefinedSubmodel {
                model_path,
                submodel,
                reference_span: _,
            } => {
                // TODO: add context "did you mean `{}`?" using hamming distance to suggest similar submodel names
                let identifier_str = submodel.as_str();
                match model_path {
                    Some(path) => {
                        let path = path.as_ref().display();
                        write!(
                            f,
                            "submodel `{identifier_str}` is not defined in model `{path}`"
                        )
                    }
                    None => write!(
                        f,
                        "submodel `{identifier_str}` is not defined in the current model"
                    ),
                }
            }
        }
    }
}

impl AsOneilError for VariableResolutionError {
    /// Returns the error message for this variable resolution error.
    fn message(&self) -> String {
        self.to_string()
    }

    /// Returns the error location within the source code for this variable resolution error.
    ///
    /// This method extracts the span information from the error variant and converts
    /// it to an `ErrorLocation` that can be used for displaying the error in context.
    ///
    /// # Arguments
    ///
    /// * `source` - The source code string to calculate the location within
    ///
    /// # Returns
    ///
    /// An `ErrorLocation` representing where the error occurred in the source code.
    fn error_location(&self, source: &str) -> Option<ErrorLocation> {
        match self {
            Self::ModelHasError {
                path: _,
                reference_span,
            }
            | Self::ParameterHasError {
                identifier: _,
                reference_span,
            }
            | Self::SubmodelResolutionFailed {
                identifier: _,
                reference_span,
            }
            | Self::UndefinedParameter {
                model_path: _,
                parameter: _,
                reference_span,
            }
            | Self::UndefinedSubmodel {
                model_path: _,
                submodel: _,
                reference_span,
            } => {
                let start_offset = reference_span.start();
                let length = reference_span.length();
                let location = ErrorLocation::from_source_and_span(source, start_offset, length);
                Some(location)
            }
        }
    }
}
