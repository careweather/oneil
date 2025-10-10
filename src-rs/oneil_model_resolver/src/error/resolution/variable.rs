use std::fmt;

use oneil_ir as ir;
use oneil_shared::{
    error::{AsOneilError, ErrorLocation},
    span::Span,
};

/// Represents an error that occurred during variable resolution within expressions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariableResolutionError {
    /// The model that should contain the variable has errors.
    ModelHasError {
        /// The path of the model that has errors
        path: ir::ModelPath,
        /// The span of where the model is referenced
        reference_span: Span,
    },
    /// The parameter that should contain the variable has errors.
    ParameterHasError {
        /// The identifier of the parameter that has errors
        identifier: ir::Identifier,
        /// The span of where the parameter is referenced
        reference_span: Span,
    },
    /// The resolution of a submodel that is referenced by a variable has failed.
    ReferenceResolutionFailed {
        /// The identifier of the reference that has errors
        identifier: ir::ReferenceName,
        /// The span of where the reference is referenced
        reference_span: Span,
    },
    /// The parameter is not defined in the current context.
    UndefinedParameter {
        /// The path of the model that contains the parameter (if None, the parameter is not defined in the current model)
        model_path: Option<ir::ModelPath>,
        /// The identifier of the parameter that is undefined
        parameter: ir::Identifier,
        /// The span of where the parameter is referenced
        reference_span: Span,
    },
    /// The reference is not defined in the current model.
    UndefinedReference {
        /// The identifier of the reference that is undefined
        reference: ir::ReferenceName,
        /// The span of where the reference is referenced
        reference_span: Span,
    },
}

impl VariableResolutionError {
    /// Creates a new error indicating that the model has errors.
    #[must_use]
    pub const fn model_has_error(model_path: ir::ModelPath, reference_span: Span) -> Self {
        Self::ModelHasError {
            path: model_path,
            reference_span,
        }
    }

    /// Creates a new error indicating that the parameter has errors.
    #[must_use]
    pub const fn parameter_has_error(identifier: ir::Identifier, reference_span: Span) -> Self {
        Self::ParameterHasError {
            identifier,
            reference_span,
        }
    }

    /// Creates a new error indicating that resolution of a submodel that is
    /// referenced by a variable has failed.
    #[must_use]
    pub const fn reference_resolution_failed(
        identifier: ir::ReferenceName,
        reference_span: Span,
    ) -> Self {
        Self::ReferenceResolutionFailed {
            identifier,
            reference_span,
        }
    }

    /// Creates a new error indicating that the parameter is undefined in the current model.
    #[must_use]
    pub const fn undefined_parameter(parameter: ir::Identifier, reference_span: Span) -> Self {
        Self::UndefinedParameter {
            model_path: None,
            parameter,
            reference_span,
        }
    }

    /// Creates a new error indicating that the parameter is undefined in a specific reference.
    #[must_use]
    pub const fn undefined_parameter_in_reference(
        reference_path: ir::ModelPath,
        parameter: ir::Identifier,
        reference_span: Span,
    ) -> Self {
        Self::UndefinedParameter {
            model_path: Some(reference_path),
            parameter,
            reference_span,
        }
    }

    /// Creates a new error indicating that the submodel is undefined in the current model.
    #[must_use]
    pub const fn undefined_reference(reference: ir::ReferenceName, reference_span: Span) -> Self {
        Self::UndefinedReference {
            reference,
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
            Self::ReferenceResolutionFailed {
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
            Self::UndefinedReference {
                reference,
                reference_span: _,
            } => {
                // TODO: add context "did you mean `{}`?" using hamming distance to suggest similar submodel names
                let identifier_str = reference.as_str();
                write!(
                    f,
                    "reference `{identifier_str}` is not defined in the current model"
                )
            }
        }
    }
}

impl AsOneilError for VariableResolutionError {
    fn message(&self) -> String {
        self.to_string()
    }

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
            | Self::ReferenceResolutionFailed {
                identifier: _,
                reference_span,
            }
            | Self::UndefinedParameter {
                model_path: _,
                parameter: _,
                reference_span,
            }
            | Self::UndefinedReference {
                reference: _,
                reference_span,
            } => {
                let location = ErrorLocation::from_source_and_span(source, *reference_span);
                Some(location)
            }
        }
    }
}
