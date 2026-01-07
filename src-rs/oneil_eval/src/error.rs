#![expect(missing_docs, reason = "this enum will be reworked in the next task")]

use std::path::PathBuf;

use oneil_shared::{
    error::{AsOneilError, Context as ErrorContext, ErrorLocation},
    span::Span,
};

use crate::value::ValueType;

#[derive(Debug, Clone, PartialEq)]
pub struct ModelError {
    pub model_path: PathBuf,
    pub error: EvalError,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EvalError {
    InvalidUnit,
    HasExponentWithUnits,
    HasIntervalExponent,
    InvalidOperation,
    InvalidType,
    ParameterHasError,
    UndefinedBuiltinValue,
    InvalidArgumentCount,
    ParameterUnitMismatch,
    UnknownUnit {
        unit_name: String,
        unit_name_span: Span,
    },
    InvalidIfExpressionType {
        expr_span: Span,
        found_type: ValueType,
    },
    MultiplePiecewiseBranchesMatch {
        param_ident: String,
        param_ident_span: Span,
        matching_branche_spans: Vec<Span>,
    },
    NoPiecewiseBranchMatch {
        param_ident: String,
        param_ident_span: Span,
    },
    BooleanCannotHaveUnit {
        expr_span: Span,
        unit_span: Span,
    },
    StringCannotHaveUnit {
        expr_span: Span,
        unit_span: Span,
    },
    InvalidContinuousLimitMinType {
        expr_span: Span,
        found_type: ValueType,
    },
    InvalidContinuousLimitMaxType {
        expr_span: Span,
        found_type: ValueType,
    },
    LimitCannotBeBoolean,
    DuplicateStringLimit,
    ExpectedStringLimit,
    ExpectedNumberLimit,
    DiscreteLimitUnitMismatch,
    ParameterValueOutsideLimits,
    ParameterUnitDoesNotMatchLimit,
    Unsupported,
}

impl AsOneilError for EvalError {
    fn message(&self) -> String {
        match self {
            Self::InvalidUnit => todo!(),
            Self::HasExponentWithUnits => todo!(),
            Self::HasIntervalExponent => todo!(),
            Self::InvalidOperation => todo!(),
            Self::InvalidType => todo!(),
            Self::ParameterHasError => todo!(),
            Self::UndefinedBuiltinValue => todo!(),
            Self::InvalidArgumentCount => todo!(),
            Self::ParameterUnitMismatch => todo!(),
            Self::UnknownUnit {
                unit_name,
                unit_name_span: _,
            } => format!("unknown unit `{unit_name}`"),
            Self::InvalidIfExpressionType {
                expr_span: _,
                found_type,
            } => {
                format!("expected a boolean value, but found a {found_type} value")
            }
            Self::MultiplePiecewiseBranchesMatch {
                param_ident,
                param_ident_span: _,
                matching_branche_spans,
            } => format!(
                "parameter `{param_ident}` has {} matching piecewise branches",
                matching_branche_spans.len()
            ),
            Self::NoPiecewiseBranchMatch {
                param_ident,
                param_ident_span: _,
            } => format!("parameter `{param_ident}` does not have a matching piecewise branch"),
            Self::BooleanCannotHaveUnit {
                expr_span: _,
                unit_span: _,
            } => "boolean value cannot have a unit".to_string(),
            Self::StringCannotHaveUnit {
                expr_span: _,
                unit_span: _,
            } => "string value cannot have a unit".to_string(),
            Self::InvalidContinuousLimitMinType {
                expr_span: _,
                found_type,
            } => format!("expected a number value, but found a {found_type} value"),
            Self::InvalidContinuousLimitMaxType {
                expr_span: _,
                found_type,
            } => format!("expected a number value, but found a {found_type} value"),
            Self::LimitCannotBeBoolean => todo!(),
            Self::DuplicateStringLimit => todo!(),
            Self::ExpectedStringLimit => todo!(),
            Self::ExpectedNumberLimit => todo!(),
            Self::DiscreteLimitUnitMismatch => todo!(),
            Self::ParameterValueOutsideLimits => todo!(),
            Self::ParameterUnitDoesNotMatchLimit => todo!(),
            Self::Unsupported => todo!(),
        }
    }

    fn error_location(&self, source: &str) -> Option<oneil_shared::error::ErrorLocation> {
        match self {
            Self::InvalidUnit => todo!(),
            Self::HasExponentWithUnits => todo!(),
            Self::HasIntervalExponent => todo!(),
            Self::InvalidOperation => todo!(),
            Self::InvalidType => todo!(),
            Self::ParameterHasError => todo!(),
            Self::UndefinedBuiltinValue => todo!(),
            Self::InvalidArgumentCount => todo!(),
            Self::ParameterUnitMismatch => todo!(),
            Self::UnknownUnit {
                unit_name: _,
                unit_name_span: location_span,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
            Self::InvalidIfExpressionType {
                expr_span: location_span,
                found_type: _,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
            Self::MultiplePiecewiseBranchesMatch {
                param_ident: _,
                param_ident_span: location_span,
                matching_branche_spans: _,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
            Self::NoPiecewiseBranchMatch {
                param_ident: _,
                param_ident_span: location_span,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
            Self::BooleanCannotHaveUnit {
                expr_span: _,
                unit_span: location_span,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
            Self::StringCannotHaveUnit {
                expr_span: _,
                unit_span: location_span,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
            Self::InvalidContinuousLimitMinType {
                expr_span: location_span,
                found_type: _,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
            Self::InvalidContinuousLimitMaxType {
                expr_span: location_span,
                found_type: _,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
            Self::LimitCannotBeBoolean => todo!(),
            Self::DuplicateStringLimit => todo!(),
            Self::ExpectedStringLimit => todo!(),
            Self::ExpectedNumberLimit => todo!(),
            Self::DiscreteLimitUnitMismatch => todo!(),
            Self::ParameterValueOutsideLimits => todo!(),
            Self::ParameterUnitDoesNotMatchLimit => todo!(),
            Self::Unsupported => todo!(),
        }
    }

    fn context(&self) -> Vec<oneil_shared::error::Context> {
        match self {
            Self::InvalidUnit => todo!(),
            Self::HasExponentWithUnits => todo!(),
            Self::HasIntervalExponent => todo!(),
            Self::InvalidOperation => todo!(),
            Self::InvalidType => todo!(),
            Self::ParameterHasError => todo!(),
            Self::UndefinedBuiltinValue => todo!(),
            Self::InvalidArgumentCount => todo!(),
            Self::ParameterUnitMismatch => todo!(),
            Self::UnknownUnit {
                unit_name: _,
                unit_name_span: _,
            } => Vec::new(),
            Self::InvalidIfExpressionType {
                expr_span: _,
                found_type: _,
            } => {
                vec![ErrorContext::Note(
                    "piecewise conditions must evaluate to a boolean value".to_string(),
                )]
            }
            Self::MultiplePiecewiseBranchesMatch {
                param_ident: _,
                param_ident_span: _,
                matching_branche_spans: _,
            } => Vec::new(),
            Self::NoPiecewiseBranchMatch {
                param_ident: _,
                param_ident_span: _,
            } => vec![ErrorContext::Note(
                "none of the piecewise branches evaluate to `true`".to_string(),
            )],
            Self::BooleanCannotHaveUnit {
                expr_span: _,
                unit_span: _,
            } => Vec::new(),
            Self::StringCannotHaveUnit {
                expr_span: _,
                unit_span: _,
            } => Vec::new(),
            Self::InvalidContinuousLimitMinType {
                expr_span: _,
                found_type: _,
            } => vec![ErrorContext::Note(
                "continuous limit minimum must evaluate to a number value".to_string(),
            )],
            Self::InvalidContinuousLimitMaxType {
                expr_span: _,
                found_type: _,
            } => vec![ErrorContext::Note(
                "continuous limit maximum must evaluate to a number value".to_string(),
            )],
            Self::LimitCannotBeBoolean => todo!(),
            Self::DuplicateStringLimit => todo!(),
            Self::ExpectedStringLimit => todo!(),
            Self::ExpectedNumberLimit => todo!(),
            Self::DiscreteLimitUnitMismatch => todo!(),
            Self::ParameterValueOutsideLimits => todo!(),
            Self::ParameterUnitDoesNotMatchLimit => todo!(),
            Self::Unsupported => todo!(),
        }
    }

    fn context_with_source(
        &self,
        source: &str,
    ) -> Vec<(
        oneil_shared::error::Context,
        Option<oneil_shared::error::ErrorLocation>,
    )> {
        match self {
            Self::InvalidUnit => todo!(),
            Self::HasExponentWithUnits => todo!(),
            Self::HasIntervalExponent => todo!(),
            Self::InvalidOperation => todo!(),
            Self::InvalidType => todo!(),
            Self::ParameterHasError => todo!(),
            Self::UndefinedBuiltinValue => todo!(),
            Self::InvalidArgumentCount => todo!(),
            Self::ParameterUnitMismatch => todo!(),
            Self::UnknownUnit {
                unit_name: _,
                unit_name_span: _,
            } => Vec::new(),
            Self::InvalidIfExpressionType {
                expr_span: _,
                found_type: _,
            } => Vec::new(),
            Self::MultiplePiecewiseBranchesMatch {
                param_ident: _,
                param_ident_span: _,
                matching_branche_spans,
            } => matching_branche_spans
                .iter()
                .map(|branch_span| {
                    (
                        ErrorContext::Note("this condition evaluates to `true`".to_string()),
                        Some(ErrorLocation::from_source_and_span(source, *branch_span)),
                    )
                })
                .collect(),
            Self::NoPiecewiseBranchMatch {
                param_ident: _,
                param_ident_span: _,
            } => Vec::new(),
            Self::BooleanCannotHaveUnit {
                expr_span,
                unit_span: _,
            } => vec![(
                ErrorContext::Note("this expression evaluates to a boolean value".to_string()),
                Some(ErrorLocation::from_source_and_span(source, *expr_span)),
            )],
            Self::StringCannotHaveUnit {
                expr_span,
                unit_span: _,
            } => vec![(
                ErrorContext::Note("this expression evaluates to a string value".to_string()),
                Some(ErrorLocation::from_source_and_span(source, *expr_span)),
            )],
            Self::InvalidContinuousLimitMinType {
                expr_span: _,
                found_type: _,
            } => Vec::new(),
            Self::InvalidContinuousLimitMaxType {
                expr_span: _,
                found_type: _,
            } => Vec::new(),
            Self::LimitCannotBeBoolean => todo!(),
            Self::DuplicateStringLimit => todo!(),
            Self::ExpectedStringLimit => todo!(),
            Self::ExpectedNumberLimit => todo!(),
            Self::DiscreteLimitUnitMismatch => todo!(),
            Self::ParameterValueOutsideLimits => todo!(),
            Self::ParameterUnitDoesNotMatchLimit => todo!(),
            Self::Unsupported => todo!(),
        }
    }
}
