#![expect(missing_docs, reason = "this enum will be reworked in the next task")]

use std::path::PathBuf;

use oneil_shared::{
    error::{AsOneilError, Context as ErrorContext, ErrorLocation},
    span::Span,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelError {
    pub model_path: PathBuf,
    pub error: EvalError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    UnknownUnit,
    InvalidIfExpressionType,
    MultiplePiecewiseBranchesMatch,
    NoPiecewiseBranchMatch,
    BooleanCannotHaveUnit { expr_span: Span, unit_span: Span },
    StringCannotHaveUnit { expr_span: Span, unit_span: Span },
    InvalidContinuousLimitMinType,
    InvalidContinuousLimitMaxType,
    LimitCannotBeBoolean,
    DuplicateStringLimit,
    ExpectedStringLimit,
    ExpectedNumberLimit,
    DiscreteLimitUnitMismatch,
    ParameterValueOutsideLimits,
    ParameterUnitDoesNotMatchLimit,
    Unsupported,
    NoNonEmptyValue,
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
            Self::UnknownUnit => todo!(),
            Self::InvalidIfExpressionType => todo!(),
            Self::MultiplePiecewiseBranchesMatch => todo!(),
            Self::NoPiecewiseBranchMatch => todo!(),
            Self::BooleanCannotHaveUnit {
                expr_span: _,
                unit_span: _,
            } => "boolean value cannot have a unit".to_string(),
            Self::StringCannotHaveUnit {
                expr_span: _,
                unit_span: _,
            } => "string value cannot have a unit".to_string(),
            Self::InvalidContinuousLimitMinType => todo!(),
            Self::InvalidContinuousLimitMaxType => todo!(),
            Self::LimitCannotBeBoolean => todo!(),
            Self::DuplicateStringLimit => todo!(),
            Self::ExpectedStringLimit => todo!(),
            Self::ExpectedNumberLimit => todo!(),
            Self::DiscreteLimitUnitMismatch => todo!(),
            Self::ParameterValueOutsideLimits => todo!(),
            Self::ParameterUnitDoesNotMatchLimit => todo!(),
            Self::Unsupported => todo!(),
            Self::NoNonEmptyValue => todo!(),
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
            Self::UnknownUnit => todo!(),
            Self::InvalidIfExpressionType => todo!(),
            Self::MultiplePiecewiseBranchesMatch => todo!(),
            Self::NoPiecewiseBranchMatch => todo!(),
            Self::BooleanCannotHaveUnit {
                expr_span,
                unit_span,
            } => Some(ErrorLocation::from_source_and_span(source, *unit_span)),
            Self::StringCannotHaveUnit {
                expr_span,
                unit_span,
            } => Some(ErrorLocation::from_source_and_span(source, *unit_span)),
            Self::InvalidContinuousLimitMinType => todo!(),
            Self::InvalidContinuousLimitMaxType => todo!(),
            Self::LimitCannotBeBoolean => todo!(),
            Self::DuplicateStringLimit => todo!(),
            Self::ExpectedStringLimit => todo!(),
            Self::ExpectedNumberLimit => todo!(),
            Self::DiscreteLimitUnitMismatch => todo!(),
            Self::ParameterValueOutsideLimits => todo!(),
            Self::ParameterUnitDoesNotMatchLimit => todo!(),
            Self::Unsupported => todo!(),
            Self::NoNonEmptyValue => todo!(),
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
            Self::UnknownUnit => todo!(),
            Self::InvalidIfExpressionType => todo!(),
            Self::MultiplePiecewiseBranchesMatch => todo!(),
            Self::NoPiecewiseBranchMatch => todo!(),
            Self::BooleanCannotHaveUnit {
                expr_span,
                unit_span,
            } => Vec::new(),
            Self::StringCannotHaveUnit {
                expr_span,
                unit_span,
            } => Vec::new(),
            Self::InvalidContinuousLimitMinType => todo!(),
            Self::InvalidContinuousLimitMaxType => todo!(),
            Self::LimitCannotBeBoolean => todo!(),
            Self::DuplicateStringLimit => todo!(),
            Self::ExpectedStringLimit => todo!(),
            Self::ExpectedNumberLimit => todo!(),
            Self::DiscreteLimitUnitMismatch => todo!(),
            Self::ParameterValueOutsideLimits => todo!(),
            Self::ParameterUnitDoesNotMatchLimit => todo!(),
            Self::Unsupported => todo!(),
            Self::NoNonEmptyValue => todo!(),
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
            Self::UnknownUnit => todo!(),
            Self::InvalidIfExpressionType => todo!(),
            Self::MultiplePiecewiseBranchesMatch => todo!(),
            Self::NoPiecewiseBranchMatch => todo!(),
            Self::BooleanCannotHaveUnit {
                expr_span,
                unit_span,
            } => vec![(
                ErrorContext::Note("this expression evaluates to a boolean value".to_string()),
                Some(ErrorLocation::from_source_and_span(source, *expr_span)),
            )],
            Self::StringCannotHaveUnit {
                expr_span,
                unit_span,
            } => vec![(
                ErrorContext::Note("this expression evaluates to a string value".to_string()),
                Some(ErrorLocation::from_source_and_span(source, *expr_span)),
            )],
            Self::InvalidContinuousLimitMinType => todo!(),
            Self::InvalidContinuousLimitMaxType => todo!(),
            Self::LimitCannotBeBoolean => todo!(),
            Self::DuplicateStringLimit => todo!(),
            Self::ExpectedStringLimit => todo!(),
            Self::ExpectedNumberLimit => todo!(),
            Self::DiscreteLimitUnitMismatch => todo!(),
            Self::ParameterValueOutsideLimits => todo!(),
            Self::ParameterUnitDoesNotMatchLimit => todo!(),
            Self::Unsupported => todo!(),
            Self::NoNonEmptyValue => todo!(),
        }
    }
}
