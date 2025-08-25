use std::fmt;

use oneil_error::{AsOneilError, Context, ErrorLocation};
use oneil_ir::{
    reference::{Identifier, ModelPath},
    span::Span,
};

/// Represents an error that occurred during submodel resolution.
///
/// This error type is used when a `use model` declaration cannot be resolved to
/// its corresponding model. This can happen when the referenced model has errors
/// or when the submodel identifier is not defined in the referenced model.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// The submodel name is a duplicate.
    DuplicateSubmodel {
        /// The identifier of the duplicate submodel
        submodel: Identifier,
        /// The span of where the original submodel is referenced
        original_span: Span,
        /// The span of where the duplicate submodel is referenced
        duplicate_span: Span,
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
    #[must_use]
    pub const fn model_has_error(model_path: ModelPath, reference_span: Span) -> Self {
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
    #[must_use]
    pub const fn undefined_submodel_in_submodel(
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

    /// Creates a new error indicating that the submodel name is a duplicate.
    ///
    /// # Arguments
    ///
    /// * `submodel` - The identifier of the duplicate submodel
    /// * `original_span` - The span of where the original submodel is referenced
    /// * `duplicate_span` - The span of where the duplicate submodel is referenced
    ///
    /// # Returns
    ///
    /// A new `SubmodelResolutionError::DuplicateSubmodel` variant.
    #[must_use]
    pub const fn duplicate_submodel(
        submodel: Identifier,
        original_span: Span,
        duplicate_span: Span,
    ) -> Self {
        Self::DuplicateSubmodel {
            submodel,
            original_span,
            duplicate_span,
        }
    }
}

impl fmt::Display for SubmodelResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ModelHasError {
                model_path: path,
                reference_span: _,
            } => {
                let path = path.as_ref().display();
                write!(f, "submodel `{path}` has errors")
            }
            Self::UndefinedSubmodel {
                parent_model_path,
                submodel,
                reference_span: _,
            } => {
                let path = parent_model_path.as_ref().display();
                let submodel_str = submodel.as_str();
                write!(
                    f,
                    "submodel `{submodel_str}` is not defined in model `{path}`"
                )
            }
            Self::DuplicateSubmodel { submodel, .. } => {
                let submodel_str = submodel.as_str();
                write!(f, "submodel `{submodel_str}` is defined multiple times")
            }
        }
    }
}

impl AsOneilError for SubmodelResolutionError {
    fn message(&self) -> String {
        self.to_string()
    }

    fn error_location(&self, source: &str) -> Option<ErrorLocation> {
        match self {
            Self::ModelHasError {
                model_path: _,
                reference_span: location_span,
            }
            | Self::UndefinedSubmodel {
                parent_model_path: _,
                submodel: _,
                reference_span: location_span,
            }
            | Self::DuplicateSubmodel {
                duplicate_span: location_span,
                ..
            } => {
                let start = location_span.start();
                let length = location_span.length();
                let location = ErrorLocation::from_source_and_span(source, start, length);
                Some(location)
            }
        }
    }

    fn context_with_source(&self, source: &str) -> Vec<(Context, Option<ErrorLocation>)> {
        match self {
            Self::DuplicateSubmodel { duplicate_span, .. } => {
                let start = duplicate_span.start();
                let length = duplicate_span.length();
                let location = ErrorLocation::from_source_and_span(source, start, length);
                vec![(
                    Context::Note("submodel is defined multiple times".to_string()),
                    Some(location),
                )]
            }
            Self::ModelHasError { .. } | Self::UndefinedSubmodel { .. } => vec![],
        }
    }
}
