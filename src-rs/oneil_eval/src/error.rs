#![expect(missing_docs, reason = "this enum will be reworked in the next task")]

use std::{fmt, path::PathBuf};

use oneil_shared::{
    error::{AsOneilError, Context as ErrorContext, ErrorLocation},
    span::Span,
};

use crate::value::{DisplayUnit, Interval, NumberType, Value, ValueType};

#[derive(Debug, Clone, PartialEq)]
pub struct ModelError {
    pub model_path: PathBuf,
    pub error: EvalError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpectedType {
    Number,
    Boolean,
    String,
    MeasuredNumber,
    NumberOrMeasuredNumber,
}

impl fmt::Display for ExpectedType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number => write!(f, "unitless number"),
            Self::Boolean => write!(f, "boolean"),
            Self::String => write!(f, "string"),
            Self::MeasuredNumber => write!(f, "number with a unit"),
            Self::NumberOrMeasuredNumber => write!(f, "number"),
        }
    }
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
    TypeMismatch {
        expected_type: ValueType,
        expected_source_span: Span,
        found_type: ValueType,
        found_span: Span,
    },
    UnitMismatch {
        expected_unit: DisplayUnit,
        expected_source_span: Span,
        found_unit: DisplayUnit,
        found_span: Span,
    },
    InvalidType {
        expected_type: ExpectedType,
        found_type: ValueType,
        found_span: Span,
    },
    InvalidNumberType {
        number_type: NumberType,
        found_number_type: NumberType,
        found_span: Span,
    },
    ExponentHasUnits {
        exponent_span: Span,
        exponent_unit: DisplayUnit,
    },
    ExponentIsInterval {
        exponent_interval: Interval,
        exponent_value_span: Span,
    },
    ParameterHasError {
        parameter_name: String,
        parameter_name_span: Span,
    },
    InvalidArgumentCount {
        function_name: String,
        function_name_span: Span,
        expected_argument_count: ExpectedArgumentCount,
        actual_argument_count: usize,
    },
    ParameterMissingUnitAnnotation {
        param_expr_span: Span,
        param_value_unit: DisplayUnit,
    },
    ParameterUnitMismatch {
        param_expr_span: Span,
        param_value_unit: DisplayUnit,
        param_unit_span: Span,
        param_unit: DisplayUnit,
    },
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
    MaxUnitDoesNotMatchMinUnit {
        max_unit: DisplayUnit,
        max_unit_span: Span,
        min_unit: DisplayUnit,
        min_unit_span: Span,
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
    DiscreteLimitUnitMismatch {
        limit_unit: DisplayUnit,
        limit_span: Span,
        value_unit: DisplayUnit,
        value_unit_span: Span,
    },
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
    StringCannotHaveNumberLimit {
        param_expr_span: Span,
        param_value: Value,
        limit_span: Span,
    },
    NumberCannotHaveStringLimit {
        param_expr_span: Span,
        param_value: Value,
        limit_span: Span,
    },
    UnitlessNumberCannotHaveLimitWithUnit {
        param_expr_span: Span,
        param_value: Value,
        limit_span: Span,
        limit_unit: DisplayUnit,
    },
    LimitUnitDoesNotMatchParameterUnit {
        param_unit: DisplayUnit,
        limit_span: Span,
        limit_unit: DisplayUnit,
    },
    Unsupported {
        relevant_span: Span,
        feature_name: Option<String>,
        will_be_supported: bool,
    },
}

impl AsOneilError for EvalError {
    #[expect(clippy::too_many_lines, reason = "matching on each enum variant")]
    #[expect(
        clippy::match_same_arms,
        reason = "in order to keep the enums in order, we don't combine the same arms"
    )]
    fn message(&self) -> String {
        match self {
            Self::TypeMismatch {
                expected_type,
                expected_source_span: _,
                found_type,
                found_span: _,
            } => format!("expected type `{expected_type}` but found `{found_type}`"),
            Self::UnitMismatch {
                expected_unit,
                expected_source_span: _,
                found_unit,
                found_span: _,
            } => format!("expected unit `{expected_unit}` but found `{found_unit}`"),
            Self::InvalidType {
                expected_type,
                found_type,
                found_span: _,
            } => format!("expected type `{expected_type}` but found `{found_type}`"),
            Self::InvalidNumberType {
                number_type,
                found_number_type,
                found_span: _,
            } => {
                let number_type = match number_type {
                    NumberType::Scalar => "scalar",
                    NumberType::Interval => "interval",
                };

                let found_number_type = match found_number_type {
                    NumberType::Scalar => "scalar",
                    NumberType::Interval => "interval",
                };

                format!("expected {number_type} but found {found_number_type}")
            }
            Self::ExponentHasUnits {
                exponent_span: _,
                exponent_unit: _,
            } => "exponent cannot have units".to_string(),
            Self::ExponentIsInterval {
                exponent_interval: _,
                exponent_value_span: _,
            } => "exponent cannot be an interval".to_string(),
            Self::ParameterHasError {
                parameter_name,
                parameter_name_span: _,
            } => {
                format!("parameter `{parameter_name}` has errors")
            }
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
            Self::ParameterMissingUnitAnnotation {
                param_expr_span: _,
                param_value_unit: _,
            } => "parameter is missing a unit".to_string(),
            Self::ParameterUnitMismatch {
                param_expr_span: _,
                param_value_unit,
                param_unit_span: _,
                param_unit,
            } => format!(
                "parameter value unit `{param_value_unit}` does not match expected unit `{param_unit}`"
            ),
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
            Self::MaxUnitDoesNotMatchMinUnit {
                max_unit: _,
                max_unit_span: _,
                min_unit: _,
                min_unit_span: _,
            } => "max limit unit does not match min limit unit".to_string(),
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
            Self::DiscreteLimitUnitMismatch {
                limit_unit,
                limit_span: _,
                value_unit,
                value_unit_span: _,
            } => {
                format!(
                    "discrete limit value unit `{value_unit}` does not match expected unit `{limit_unit}`"
                )
            }
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
            } => "boolean value cannot have a limit".to_string(),
            Self::StringCannotHaveNumberLimit {
                param_expr_span: _,
                param_value: _,
                limit_span: _,
            } => "string value cannot have a number limit".to_string(),
            Self::NumberCannotHaveStringLimit {
                param_expr_span: _,
                param_value: _,
                limit_span: _,
            } => "number value cannot have a string limit".to_string(),
            Self::UnitlessNumberCannotHaveLimitWithUnit {
                param_expr_span: _,
                param_value: _,
                limit_span: _,
                limit_unit,
            } => {
                format!("unitless number cannot have a limit with unit `{limit_unit}`")
            }
            Self::LimitUnitDoesNotMatchParameterUnit {
                param_unit,
                limit_span: _,
                limit_unit,
            } => {
                format!("limit unit `{limit_unit}` does not match parameter unit `{param_unit}`")
            }
            Self::Unsupported {
                relevant_span: _,
                feature_name,
                will_be_supported: _,
            } => feature_name.as_ref().map_or_else(
                || "unsupported feature".to_string(),
                |feature_name| format!("unsupported feature: `{feature_name}`"),
            ),
        }
    }

    #[expect(clippy::too_many_lines, reason = "matching on each enum variant")]
    fn error_location(&self, source: &str) -> Option<ErrorLocation> {
        match self {
            Self::TypeMismatch {
                expected_type: _,
                expected_source_span: _,
                found_type: _,
                found_span: location_span,
            }
            | Self::UnitMismatch {
                expected_unit: _,
                expected_source_span: _,
                found_unit: _,
                found_span: location_span,
            }
            | Self::InvalidType {
                expected_type: _,
                found_type: _,
                found_span: location_span,
            }
            | Self::InvalidNumberType {
                number_type: _,
                found_number_type: _,
                found_span: location_span,
            }
            | Self::ExponentHasUnits {
                exponent_span: location_span,
                exponent_unit: _,
            }
            | Self::ExponentIsInterval {
                exponent_interval: _,
                exponent_value_span: location_span,
            }
            | Self::ParameterHasError {
                parameter_name: _,
                parameter_name_span: location_span,
            }
            | Self::InvalidArgumentCount {
                function_name: _,
                function_name_span: location_span,
                expected_argument_count: _,
                actual_argument_count: _,
            }
            | Self::ParameterMissingUnitAnnotation {
                param_expr_span: location_span,
                param_value_unit: _,
            }
            | Self::ParameterUnitMismatch {
                param_expr_span: location_span,
                param_value_unit: _,
                param_unit_span: _,
                param_unit: _,
            }
            | Self::UnknownUnit {
                unit_name: _,
                unit_name_span: location_span,
            }
            | Self::InvalidIfExpressionType {
                expr_span: location_span,
                found_value: _,
            }
            | Self::MultiplePiecewiseBranchesMatch {
                param_ident: _,
                param_ident_span: location_span,
                matching_branche_spans: _,
            }
            | Self::NoPiecewiseBranchMatch {
                param_ident: _,
                param_ident_span: location_span,
            }
            | Self::BooleanCannotHaveUnit {
                expr_span: _,
                unit_span: location_span,
            }
            | Self::StringCannotHaveUnit {
                expr_span: _,
                unit_span: location_span,
            }
            | Self::InvalidContinuousLimitMinType {
                expr_span: location_span,
                found_value: _,
            }
            | Self::InvalidContinuousLimitMaxType {
                expr_span: location_span,
                found_value: _,
            }
            | Self::MaxUnitDoesNotMatchMinUnit {
                max_unit: _,
                max_unit_span: location_span,
                min_unit: _,
                min_unit_span: _,
            }
            | Self::BooleanCannotBeDiscreteLimitValue {
                expr_span: location_span,
            }
            | Self::DuplicateStringLimit {
                expr_span: location_span,
                original_expr_span: _,
                string_value: _,
            }
            | Self::ExpectedStringLimit {
                expr_span: location_span,
                found_value: _,
            }
            | Self::ExpectedNumberLimit {
                expr_span: location_span,
                found_value: _,
            }
            | Self::DiscreteLimitUnitMismatch {
                limit_unit: _,
                limit_span: _,
                value_unit: _,
                value_unit_span: location_span,
            }
            | Self::ParameterValueBelowDefaultLimits {
                param_expr_span: location_span,
                param_value: _,
            }
            | Self::ParameterValueBelowContinuousLimits {
                param_expr_span: location_span,
                param_value: _,
                min_expr_span: _,
                min_value: _,
            }
            | Self::ParameterValueAboveContinuousLimits {
                param_expr_span: location_span,
                param_value: _,
                max_expr_span: _,
                max_value: _,
            }
            | Self::ParameterValueNotInDiscreteLimits {
                param_expr_span: location_span,
                param_value: _,
                limit_expr_span: _,
                limit_values: _,
            }
            | Self::BooleanCannotHaveALimit {
                expr_span: location_span,
                limit_span: _,
            }
            | Self::StringCannotHaveNumberLimit {
                param_expr_span: _,
                param_value: _,
                limit_span: location_span,
            }
            | Self::NumberCannotHaveStringLimit {
                param_expr_span: _,
                param_value: _,
                limit_span: location_span,
            }
            | Self::UnitlessNumberCannotHaveLimitWithUnit {
                param_expr_span: _,
                param_value: _,
                limit_span: location_span,
                limit_unit: _,
            }
            | Self::LimitUnitDoesNotMatchParameterUnit {
                param_unit: _,
                limit_span: location_span,
                limit_unit: _,
            }
            | Self::Unsupported {
                relevant_span: location_span,
                feature_name: _,
                will_be_supported: _,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
        }
    }

    #[expect(clippy::too_many_lines, reason = "matching on each enum variant")]
    #[expect(
        clippy::match_same_arms,
        reason = "in order to keep the enums in order, we don't combine the same arms"
    )]
    fn context(&self) -> Vec<ErrorContext> {
        match self {
            Self::TypeMismatch {
                expected_type: _,
                expected_source_span: _,
                found_type: _,
                found_span: _,
            } => Vec::new(),
            Self::UnitMismatch {
                expected_unit: _,
                expected_source_span: _,
                found_unit: _,
                found_span: _,
            } => Vec::new(),
            Self::InvalidType {
                expected_type: _,
                found_type: _,
                found_span: _,
            } => Vec::new(),
            Self::InvalidNumberType {
                number_type: _,
                found_number_type: _,
                found_span: _,
            } => Vec::new(),
            Self::ExponentHasUnits {
                exponent_span: _,
                exponent_unit,
            } => {
                vec![ErrorContext::Note(format!(
                    "exponent unit is `{exponent_unit}`"
                ))]
            }
            Self::ExponentIsInterval {
                exponent_interval,
                exponent_value_span: _,
            } => vec![ErrorContext::Note(format!(
                "exponent evaluated to interval <{} | {}>",
                exponent_interval.min(),
                exponent_interval.max(),
            ))],
            Self::ParameterHasError {
                parameter_name: _,
                parameter_name_span: _,
            } => Vec::new(),
            Self::InvalidArgumentCount {
                function_name: _,
                function_name_span: _,
                expected_argument_count: _,
                actual_argument_count: _,
            } => Vec::new(),
            Self::ParameterMissingUnitAnnotation {
                param_expr_span: _,
                param_value_unit,
            } => vec![
                ErrorContext::Note(format!("parameter value has unit `{param_value_unit}`")),
                ErrorContext::Help(format!(
                    "add a unit annotation `:{param_value_unit}` to the parameter"
                )),
            ],
            Self::ParameterUnitMismatch {
                param_expr_span: _,
                param_value_unit: _,
                param_unit_span: _,
                param_unit: _,
            } => Vec::new(),
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
            Self::MaxUnitDoesNotMatchMinUnit {
                max_unit,
                max_unit_span: _,
                min_unit: _,
                min_unit_span: _,
            } => vec![ErrorContext::Note(format!(
                "max limit unit is `{max_unit}`"
            ))],
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
            Self::DiscreteLimitUnitMismatch {
                limit_unit: _,
                limit_span: _,
                value_unit: _,
                value_unit_span: _,
            } => Vec::new(),
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
            Self::StringCannotHaveNumberLimit {
                param_expr_span: _,
                param_value: _,
                limit_span: _,
            } => Vec::new(),
            Self::NumberCannotHaveStringLimit {
                param_expr_span: _,
                param_value: _,
                limit_span: _,
            } => Vec::new(),
            Self::UnitlessNumberCannotHaveLimitWithUnit {
                param_expr_span: _,
                param_value: _,
                limit_span: _,
                limit_unit,
            } => vec![ErrorContext::Note(format!("limit has unit `{limit_unit}`"))],
            Self::LimitUnitDoesNotMatchParameterUnit {
                param_unit: _,
                limit_span: _,
                limit_unit: _,
            } => Vec::new(),
            Self::Unsupported {
                relevant_span: _,
                feature_name,
                will_be_supported,
            } => {
                if let Some(feature_name) = feature_name
                    && *will_be_supported
                {
                    vec![ErrorContext::Note(format!(
                        "`{feature_name}` will be supported in the future"
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

    #[expect(clippy::too_many_lines, reason = "matching on each enum variant")]
    #[expect(
        clippy::match_same_arms,
        reason = "in order to keep the enums in order, we don't combine the same arms"
    )]
    fn context_with_source(&self, source: &str) -> Vec<(ErrorContext, Option<ErrorLocation>)> {
        match self {
            Self::TypeMismatch {
                expected_type,
                expected_source_span,
                found_type: _,
                found_span: _,
            } => vec![(
                ErrorContext::Note(format!(
                    "expected because this expression has type `{expected_type}`",
                )),
                Some(ErrorLocation::from_source_and_span(
                    source,
                    *expected_source_span,
                )),
            )],
            Self::UnitMismatch {
                expected_unit,
                expected_source_span,
                found_unit: _,
                found_span: _,
            } => vec![(
                ErrorContext::Note(format!(
                    "expected because this expression has unit `{expected_unit}`",
                )),
                Some(ErrorLocation::from_source_and_span(
                    source,
                    *expected_source_span,
                )),
            )],
            Self::InvalidType {
                expected_type: _,
                found_type: _,
                found_span: _,
            } => Vec::new(),
            Self::InvalidNumberType {
                number_type: _,
                found_number_type: _,
                found_span: _,
            } => Vec::new(),
            Self::ExponentHasUnits {
                exponent_span: _,
                exponent_unit: _,
            } => Vec::new(),
            Self::ExponentIsInterval {
                exponent_interval: _,
                exponent_value_span: _,
            } => Vec::new(),
            Self::ParameterHasError {
                parameter_name: _,
                parameter_name_span: _,
            } => Vec::new(),
            Self::InvalidArgumentCount {
                function_name: _,
                function_name_span: _,
                expected_argument_count: _,
                actual_argument_count: _,
            } => Vec::new(),
            Self::ParameterMissingUnitAnnotation {
                param_expr_span: _,
                param_value_unit: _,
            } => Vec::new(),
            Self::ParameterUnitMismatch {
                param_expr_span: _,
                param_value_unit: _,
                param_unit_span,
                param_unit: _,
            } => vec![(
                ErrorContext::Note("parameter unit defined here".to_string()),
                Some(ErrorLocation::from_source_and_span(
                    source,
                    *param_unit_span,
                )),
            )],
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
            Self::MaxUnitDoesNotMatchMinUnit {
                max_unit: _,
                max_unit_span: _,
                min_unit,
                min_unit_span,
            } => vec![(
                ErrorContext::Note(format!("min limit unit is `{min_unit}`")),
                Some(ErrorLocation::from_source_and_span(source, *min_unit_span)),
            )],
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
            Self::DiscreteLimitUnitMismatch {
                limit_unit: _,
                limit_span,
                value_unit: _,
                value_unit_span: _,
            } => vec![(
                ErrorContext::Note("expected unit was derived from this expression".to_string()),
                Some(ErrorLocation::from_source_and_span(source, *limit_span)),
            )],
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
                    .iter()
                    .map(Value::to_string)
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
            Self::StringCannotHaveNumberLimit {
                param_expr_span,
                param_value,
                limit_span: _,
            } => vec![(
                ErrorContext::Note(format!("parameter value is {param_value}")),
                Some(ErrorLocation::from_source_and_span(
                    source,
                    *param_expr_span,
                )),
            )],
            Self::NumberCannotHaveStringLimit {
                param_expr_span,
                param_value,
                limit_span: _,
            } => vec![(
                ErrorContext::Note(format!("parameter value is {param_value}")),
                Some(ErrorLocation::from_source_and_span(
                    source,
                    *param_expr_span,
                )),
            )],
            Self::UnitlessNumberCannotHaveLimitWithUnit {
                param_expr_span: _,
                param_value: _,
                limit_span: _,
                limit_unit: _,
            } => Vec::new(),
            Self::LimitUnitDoesNotMatchParameterUnit {
                param_unit: _,
                limit_span: _,
                limit_unit: _,
            } => Vec::new(),
            Self::Unsupported {
                relevant_span: _,
                feature_name: _,
                will_be_supported: _,
            } => Vec::new(),
        }
    }
}
