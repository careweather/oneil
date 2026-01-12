#![expect(missing_docs, reason = "this enum will be reworked in the next task")]

use std::path::PathBuf;

use oneil_shared::{
    error::{AsOneilError, Context as ErrorContext, ErrorLocation},
    span::Span,
};

use crate::value::{DisplayUnit, Value};

#[derive(Debug, Clone, PartialEq)]
pub struct ModelError {
    pub model_path: PathBuf,
    pub error: EvalError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpectedArgumentCount {
    Exact(usize),
    AtLeast(usize),
    AtMost(usize),
    Between(usize, usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum EvalError {
    InvalidUnit,
    HasExponentWithUnits {
        exponent_span: Span,
        exponent_value: Value,
    },
    HasIntervalExponent,
    InvalidOperation,
    InvalidType,
    ParameterHasError {
        parameter_name: String,
        parameter_name_span: Span,
    },
    UndefinedBuiltinValue,
    InvalidArgumentCount {
        function_name: String,
        function_name_span: Span,
        expected_argument_count: ExpectedArgumentCount,
        actual_argument_count: usize,
    },
    ParameterUnitMismatch,
    UnknownUnit {
        unit_name: String,
        unit_name_span: Span,
    },
    InvalidIfExpressionType {
        expr_span: Span,
        found_value: Value,
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
        found_value: Value,
    },
    InvalidContinuousLimitMaxType {
        expr_span: Span,
        found_value: Value,
    },
    BooleanCannotBeDiscreteLimitValue {
        expr_span: Span,
    },
    DuplicateStringLimit {
        expr_span: Span,
        original_expr_span: Span,
        string_value: String,
    },
    ExpectedStringLimit {
        expr_span: Span,
        found_value: Value,
    },
    ExpectedNumberLimit {
        expr_span: Span,
        found_value: Value,
    },
    DiscreteLimitUnitMismatch,
    ParameterValueBelowDefaultLimits {
        param_expr_span: Span,
        param_value: Value,
    },
    ParameterValueBelowContinuousLimits {
        param_expr_span: Span,
        param_value: Value,
        min_expr_span: Span,
        min_value: Value,
    },
    ParameterValueAboveContinuousLimits {
        param_expr_span: Span,
        param_value: Value,
        max_expr_span: Span,
        max_value: Value,
    },
    ParameterValueNotInDiscreteLimits {
        param_expr_span: Span,
        param_value: Value,
        limit_expr_span: Span,
        limit_values: Vec<Value>,
    },
    BooleanCannotHaveALimit {
        expr_span: Span,
        limit_span: Span,
    },
    ParameterUnitDoesNotMatchLimit,
    Unsupported {
        relevant_span: Span,
        feature_name: Option<String>,
        will_be_supported: bool,
    },
}

impl AsOneilError for EvalError {
    fn message(&self) -> String {
        match self {
            Self::InvalidUnit => todo!(),
            Self::HasExponentWithUnits {
                exponent_span: _,
                exponent_value: _,
            } => format!("exponent cannot have units"),
            Self::HasIntervalExponent => todo!(),
            Self::InvalidOperation => todo!(),
            Self::InvalidType => todo!(),
            Self::ParameterHasError {
                parameter_name,
                parameter_name_span: _,
            } => {
                format!("parameter `{parameter_name}` has errors")
            }
            Self::UndefinedBuiltinValue => todo!(),
            Self::InvalidArgumentCount {
                function_name,
                function_name_span: _,
                expected_argument_count,
                actual_argument_count,
            } => {
                let expected_argument_count = match expected_argument_count {
                    ExpectedArgumentCount::Exact(count) if *count == 1 => "1 argument".to_string(),
                    ExpectedArgumentCount::Exact(count) => format!("{count} arguments"),
                    ExpectedArgumentCount::AtLeast(count) if *count == 1 => {
                        "at least 1 argument".to_string()
                    }
                    ExpectedArgumentCount::AtLeast(count) => format!("at least {count} arguments"),
                    ExpectedArgumentCount::AtMost(count) if *count == 1 => {
                        "at most 1 argument".to_string()
                    }
                    ExpectedArgumentCount::AtMost(count) => format!("at most {count} arguments"),
                    ExpectedArgumentCount::Between(min, max) => {
                        format!("between {min} and {max} arguments")
                    }
                };

                format!(
                    "{function_name} expects {expected_argument_count}, but found {actual_argument_count}"
                )
            }
            Self::ParameterUnitMismatch => todo!(),
            Self::UnknownUnit {
                unit_name,
                unit_name_span: _,
            } => format!("unknown unit `{unit_name}`"),
            Self::InvalidIfExpressionType {
                expr_span: _,
                found_value,
            } => {
                format!("expected a boolean value, but found {found_value}")
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
                found_value,
            } => {
                format!("expected a number value, but found {found_value}")
            }
            Self::InvalidContinuousLimitMaxType {
                expr_span: _,
                found_value,
            } => {
                format!("expected a number value, but found {found_value}")
            }
            Self::BooleanCannotBeDiscreteLimitValue { expr_span: _ } => {
                "discrete limit cannot contain a boolean value".to_string()
            }
            Self::DuplicateStringLimit {
                expr_span: _,
                original_expr_span: _,
                string_value,
            } => format!("duplicate string value '{string_value}' in discrete limit"),
            Self::ExpectedStringLimit {
                expr_span: _,
                found_value,
            } => {
                format!("expected a string value, but found {found_value}")
            }
            Self::ExpectedNumberLimit {
                expr_span: _,
                found_value,
            } => {
                format!("expected a number value, but found {found_value}")
            }
            Self::DiscreteLimitUnitMismatch => todo!(),
            Self::ParameterValueBelowDefaultLimits {
                param_expr_span: _,
                param_value,
            } => {
                format!("parameter value {param_value} is below the default parameter limit")
            }
            Self::ParameterValueBelowContinuousLimits {
                param_expr_span: _,
                param_value,
                min_expr_span: _,
                min_value: _,
            } => {
                format!("parameter value {param_value} is below the limit")
            }
            Self::ParameterValueAboveContinuousLimits {
                param_expr_span: _,
                param_value,
                max_expr_span: _,
                max_value: _,
            } => {
                format!("parameter value {param_value} is above the limit")
            }
            Self::ParameterValueNotInDiscreteLimits {
                param_expr_span: _,
                param_value,
                limit_expr_span: _,
                limit_values: _,
            } => {
                format!("parameter value {param_value} is not in the discrete limit")
            }
            Self::BooleanCannotHaveALimit {
                expr_span: _,
                limit_span: _,
            } => {
                format!("boolean value cannot have a limit")
            }
            Self::ParameterUnitDoesNotMatchLimit => todo!(),
            Self::Unsupported {
                relevant_span: _,
                feature_name,
                will_be_supported: _,
            } => {
                if let Some(feature_name) = feature_name {
                    format!("unsupported feature: {feature_name}")
                } else {
                    "unsupported feature".to_string()
                }
            }
        }
    }

    fn error_location(&self, source: &str) -> Option<oneil_shared::error::ErrorLocation> {
        match self {
            Self::InvalidUnit => todo!(),
            Self::HasExponentWithUnits {
                exponent_span: location_span,
                exponent_value: _,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
            Self::HasIntervalExponent => todo!(),
            Self::InvalidOperation => todo!(),
            Self::InvalidType => todo!(),
            Self::ParameterHasError {
                parameter_name: _,
                parameter_name_span: location_span,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
            Self::UndefinedBuiltinValue => todo!(),
            Self::InvalidArgumentCount {
                function_name: _,
                function_name_span: location_span,
                expected_argument_count: _,
                actual_argument_count: _,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
            Self::ParameterUnitMismatch => todo!(),
            Self::UnknownUnit {
                unit_name: _,
                unit_name_span: location_span,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
            Self::InvalidIfExpressionType {
                expr_span: location_span,
                found_value: _,
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
                found_value: _,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
            Self::InvalidContinuousLimitMaxType {
                expr_span: location_span,
                found_value: _,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
            Self::BooleanCannotBeDiscreteLimitValue {
                expr_span: location_span,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
            Self::DuplicateStringLimit {
                expr_span: location_span,
                original_expr_span: _,
                string_value: _,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
            Self::ExpectedStringLimit {
                expr_span: location_span,
                found_value: _,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
            Self::ExpectedNumberLimit {
                expr_span: location_span,
                found_value: _,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
            Self::DiscreteLimitUnitMismatch => todo!(),
            Self::ParameterValueBelowDefaultLimits {
                param_expr_span: location_span,
                param_value: _,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
            Self::ParameterValueBelowContinuousLimits {
                param_expr_span: location_span,
                param_value: _,
                min_expr_span: _,
                min_value: _,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
            Self::ParameterValueAboveContinuousLimits {
                param_expr_span: location_span,
                param_value: _,
                max_expr_span: _,
                max_value: _,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
            Self::ParameterValueNotInDiscreteLimits {
                param_expr_span: location_span,
                param_value: _,
                limit_expr_span: _,
                limit_values: _,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
            Self::BooleanCannotHaveALimit {
                expr_span: location_span,
                limit_span: _,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
            Self::ParameterUnitDoesNotMatchLimit => todo!(),
            Self::Unsupported {
                relevant_span: location_span,
                feature_name: _,
                will_be_supported: _,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
        }
    }

    fn context(&self) -> Vec<oneil_shared::error::Context> {
        match self {
            Self::InvalidUnit => todo!(),
            Self::HasExponentWithUnits {
                exponent_span: _,
                exponent_value,
            } => {
                vec![ErrorContext::Note(format!(
                    "exponent evaluated to {exponent_value}"
                ))]
            }
            Self::HasIntervalExponent => todo!(),
            Self::InvalidOperation => todo!(),
            Self::InvalidType => todo!(),
            Self::ParameterHasError {
                parameter_name: _,
                parameter_name_span: _,
            } => Vec::new(),
            Self::UndefinedBuiltinValue => todo!(),
            Self::InvalidArgumentCount {
                function_name: _,
                function_name_span: _,
                expected_argument_count: _,
                actual_argument_count: _,
            } => Vec::new(),
            Self::ParameterUnitMismatch => todo!(),
            Self::UnknownUnit {
                unit_name: _,
                unit_name_span: _,
            } => Vec::new(),
            Self::InvalidIfExpressionType {
                expr_span: _,
                found_value: _,
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
                found_value: _,
            } => vec![ErrorContext::Note(
                "continuous limit minimum must evaluate to a number value".to_string(),
            )],
            Self::InvalidContinuousLimitMaxType {
                expr_span: _,
                found_value: _,
            } => vec![ErrorContext::Note(
                "continuous limit maximum must evaluate to a number value".to_string(),
            )],
            Self::BooleanCannotBeDiscreteLimitValue { expr_span: _ } => vec![ErrorContext::Note(
                "discrete limit values must evaluate to a number or a string value".to_string(),
            )],
            Self::DuplicateStringLimit {
                expr_span: _,
                original_expr_span: _,
                string_value: _,
            } => vec![ErrorContext::Note(
                "duplicate string values in discrete limits are not allowed".to_string(),
            )],
            Self::ExpectedStringLimit {
                expr_span: _,
                found_value: _,
            } => Vec::new(),
            Self::ExpectedNumberLimit {
                expr_span: _,
                found_value: _,
            } => Vec::new(),
            Self::DiscreteLimitUnitMismatch => todo!(),
            Self::ParameterValueBelowDefaultLimits {
                param_expr_span: _,
                param_value: _,
            } => vec![
                ErrorContext::Note(
                    "the default limits for number parameters are (0, inf)".to_string(),
                ),
                ErrorContext::Help(
                    "consider specifying a continuous limit for the parameter".to_string(),
                ),
            ],
            Self::ParameterValueBelowContinuousLimits {
                param_expr_span: _,
                param_value: _,
                min_expr_span: _,
                min_value: _,
            } => Vec::new(),
            Self::ParameterValueAboveContinuousLimits {
                param_expr_span: _,
                param_value: _,
                max_expr_span: _,
                max_value: _,
            } => Vec::new(),
            Self::ParameterValueNotInDiscreteLimits {
                param_expr_span: _,
                param_value: _,
                limit_expr_span: _,
                limit_values: _,
            } => Vec::new(),
            Self::BooleanCannotHaveALimit {
                expr_span: _,
                limit_span: _,
            } => Vec::new(),
            Self::ParameterUnitDoesNotMatchLimit => todo!(),
            Self::Unsupported {
                relevant_span: _,
                feature_name,
                will_be_supported,
            } => {
                if let Some(feature_name) = feature_name
                    && *will_be_supported
                {
                    vec![ErrorContext::Note(format!(
                        "{feature_name} will be supported in the future"
                    ))]
                } else if *will_be_supported {
                    vec![ErrorContext::Note(
                        "this feature will be supported in the future".to_string(),
                    )]
                } else {
                    Vec::new()
                }
            }
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
            Self::HasExponentWithUnits {
                exponent_span: _,
                exponent_value: _,
            } => Vec::new(),
            Self::HasIntervalExponent => todo!(),
            Self::InvalidOperation => todo!(),
            Self::InvalidType => todo!(),
            Self::ParameterHasError {
                parameter_name: _,
                parameter_name_span: _,
            } => Vec::new(),
            Self::UndefinedBuiltinValue => todo!(),
            Self::InvalidArgumentCount {
                function_name: _,
                function_name_span: _,
                expected_argument_count: _,
                actual_argument_count: _,
            } => Vec::new(),
            Self::ParameterUnitMismatch => todo!(),
            Self::UnknownUnit {
                unit_name: _,
                unit_name_span: _,
            } => Vec::new(),
            Self::InvalidIfExpressionType {
                expr_span: _,
                found_value: _,
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
                found_value: _,
            } => Vec::new(),
            Self::InvalidContinuousLimitMaxType {
                expr_span: _,
                found_value: _,
            } => Vec::new(),
            Self::BooleanCannotBeDiscreteLimitValue { expr_span: _ } => Vec::new(),
            Self::DuplicateStringLimit {
                expr_span: _,
                original_expr_span,
                string_value,
            } => vec![(
                ErrorContext::Note(format!("original value `{string_value}` is found here")),
                Some(ErrorLocation::from_source_and_span(
                    source,
                    *original_expr_span,
                )),
            )],
            Self::ExpectedStringLimit {
                expr_span: _,
                found_value: _,
            } => Vec::new(),
            Self::ExpectedNumberLimit {
                expr_span: _,
                found_value: _,
            } => Vec::new(),
            Self::DiscreteLimitUnitMismatch => todo!(),
            Self::ParameterValueBelowDefaultLimits {
                param_expr_span: _,
                param_value: _,
            } => Vec::new(),
            Self::ParameterValueBelowContinuousLimits {
                param_expr_span: _,
                param_value: _,
                min_expr_span,
                min_value,
            } => vec![(
                ErrorContext::Note(format!(
                    "the limit minimum for this parameter is {min_value}"
                )),
                Some(ErrorLocation::from_source_and_span(source, *min_expr_span)),
            )],
            Self::ParameterValueAboveContinuousLimits {
                param_expr_span: _,
                param_value: _,
                max_expr_span,
                max_value,
            } => vec![(
                ErrorContext::Note(format!(
                    "the limit maximum for this parameter is {max_value}"
                )),
                Some(ErrorLocation::from_source_and_span(source, *max_expr_span)),
            )],
            Self::ParameterValueNotInDiscreteLimits {
                param_expr_span: _,
                param_value: _,
                limit_expr_span,
                limit_values,
            } => {
                let limit_values = limit_values
                    .into_iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                vec![(
                    ErrorContext::Note(format!(
                        "the limit values for this parameter are [{limit_values}]"
                    )),
                    Some(ErrorLocation::from_source_and_span(
                        source,
                        *limit_expr_span,
                    )),
                )]
            }
            Self::BooleanCannotHaveALimit {
                expr_span: _,
                limit_span: _,
            } => Vec::new(),
            Self::ParameterUnitDoesNotMatchLimit => todo!(),
            Self::Unsupported {
                relevant_span: _,
                feature_name: _,
                will_be_supported: _,
            } => Vec::new(),
        }
    }
}
