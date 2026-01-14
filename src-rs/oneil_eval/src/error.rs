#![expect(missing_docs, reason = "this enum will be reworked in the next task")]

use std::path::PathBuf;

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
pub enum NumberBinaryOperation {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Pow,
    MinMax,
    LessThan,
    LessThanEq,
    GreaterThan,
    GreaterThanEq,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanBinaryOperation {
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperation {
    Number(NumberBinaryOperation),
    Boolean(BooleanBinaryOperation),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryEvalError {
    UnitMismatch {
        lhs_unit: DisplayUnit,
        rhs_unit: DisplayUnit,
    },
    TypeMismatch {
        lhs_type: ValueType,
        rhs_type: ValueType,
    },
    InvalidType {
        op: BinaryOperation,
        lhs_type: ValueType,
    },
    ExponentHasUnits {
        exponent_unit: DisplayUnit,
    },
    ExponentIsInterval {
        exponent_interval: Interval,
    },
    InvalidExponentType {
        exponent_type: ValueType,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperation {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryEvalError {
    InvalidType {
        op: UnaryOperation,
        value_type: ValueType,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpectedType {
    Number,
    Boolean,
    String,
    MeasuredNumber,
    NumberOrMeasuredNumber,
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
    BinaryEvalError {
        lhs_span: Span,
        rhs_span: Span,
        error: BinaryEvalError,
    },
    UnaryEvalError {
        expr_span: Span,
        error: UnaryEvalError,
    },
    HasExponentWithUnits {
        exponent_span: Span,
        exponent_value: Value,
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
    fn message(&self) -> String {
        match self {
            Self::BinaryEvalError {
                lhs_span: _,
                rhs_span: _,
                error,
            } => match error {
                BinaryEvalError::UnitMismatch { lhs_unit, rhs_unit } => {
                    format!("expected unit `{rhs_unit}` but found `{lhs_unit}`")
                }
                BinaryEvalError::TypeMismatch { lhs_type, rhs_type } => {
                    format!("expected type `{rhs_type}` but found `{lhs_type}`")
                }
                BinaryEvalError::InvalidType { op, lhs_type } => {
                    let op_str = match op {
                        BinaryOperation::Number(NumberBinaryOperation::Add) => "addition",
                        BinaryOperation::Number(NumberBinaryOperation::Sub) => "subtraction",
                        BinaryOperation::Number(NumberBinaryOperation::Mul) => "multiplication",
                        BinaryOperation::Number(NumberBinaryOperation::Div) => "division",
                        BinaryOperation::Number(NumberBinaryOperation::Rem) => "remainder",
                        BinaryOperation::Number(NumberBinaryOperation::Pow) => "power",
                        BinaryOperation::Number(NumberBinaryOperation::MinMax) => "min/max",
                        BinaryOperation::Number(NumberBinaryOperation::LessThan) => "less than",
                        BinaryOperation::Number(NumberBinaryOperation::LessThanEq) => {
                            "less than or equal to"
                        }
                        BinaryOperation::Number(NumberBinaryOperation::GreaterThan) => {
                            "greater than"
                        }
                        BinaryOperation::Number(NumberBinaryOperation::GreaterThanEq) => {
                            "greater than or equal to"
                        }
                        BinaryOperation::Boolean(BooleanBinaryOperation::And) => "and",
                        BinaryOperation::Boolean(BooleanBinaryOperation::Or) => "or",
                    };

                    let expected_type = match op {
                        BinaryOperation::Number(_) => "number",
                        BinaryOperation::Boolean(_) => "boolean",
                    };

                    format!("'{op_str}' operation expects a {expected_type} but found `{lhs_type}`")
                }
                BinaryEvalError::ExponentHasUnits { exponent_unit: _ } => {
                    format!("exponent cannot have units")
                }
                BinaryEvalError::ExponentIsInterval {
                    exponent_interval: _,
                } => {
                    format!("exponent cannot be an interval")
                }
                BinaryEvalError::InvalidExponentType { exponent_type } => {
                    format!("expected a number exponent but found `{exponent_type}`")
                }
            },
            Self::UnaryEvalError { expr_span, error } => match error {
                UnaryEvalError::InvalidType { op, value_type } => {
                    let op_str = match op {
                        UnaryOperation::Neg => "negation",
                        UnaryOperation::Not => "logical NOT",
                    };
                    let expected_type = match op {
                        UnaryOperation::Neg => "number",
                        UnaryOperation::Not => "boolean",
                    };

                    format!(
                        "'{op_str}' operation expects a {expected_type} but found `{value_type}`"
                    )
                }
            },
            Self::HasExponentWithUnits {
                exponent_span: _,
                exponent_value: _,
            } => format!("exponent cannot have units"),
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
            } => {
                format!("parameter is missing a unit")
            }
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
            } => format!("max limit unit does not match min limit unit"),
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
            } => {
                format!("boolean value cannot have a limit")
            }
            Self::StringCannotHaveNumberLimit {
                param_expr_span: _,
                param_value: _,
                limit_span: _,
            } => {
                format!("string value cannot have a number limit")
            }
            Self::NumberCannotHaveStringLimit {
                param_expr_span: _,
                param_value: _,
                limit_span: _,
            } => {
                format!("number value cannot have a string limit")
            }
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
            } => {
                if let Some(feature_name) = feature_name {
                    format!("unsupported feature: `{feature_name}`")
                } else {
                    "unsupported feature".to_string()
                }
            }
        }
    }

    fn error_location(&self, source: &str) -> Option<ErrorLocation> {
        match self {
            Self::BinaryEvalError {
                lhs_span,
                rhs_span,
                error,
            } => match error {
                BinaryEvalError::UnitMismatch {
                    lhs_unit: _,
                    rhs_unit: _,
                } => Some(ErrorLocation::from_source_and_span(source, *rhs_span)),
                BinaryEvalError::TypeMismatch {
                    lhs_type: _,
                    rhs_type: _,
                } => Some(ErrorLocation::from_source_and_span(source, *rhs_span)),
                BinaryEvalError::InvalidType { op: _, lhs_type: _ } => {
                    Some(ErrorLocation::from_source_and_span(source, *lhs_span))
                }
                BinaryEvalError::ExponentHasUnits { exponent_unit: _ } => {
                    Some(ErrorLocation::from_source_and_span(source, *rhs_span))
                }
                BinaryEvalError::ExponentIsInterval {
                    exponent_interval: _,
                } => Some(ErrorLocation::from_source_and_span(source, *rhs_span)),
                BinaryEvalError::InvalidExponentType { exponent_type: _ } => {
                    Some(ErrorLocation::from_source_and_span(source, *rhs_span))
                }
            },
            Self::UnaryEvalError { expr_span, error } => match error {
                UnaryEvalError::InvalidType {
                    op: _,
                    value_type: _,
                } => Some(ErrorLocation::from_source_and_span(source, *expr_span)),
            },
            Self::HasExponentWithUnits {
                exponent_span: location_span,
                exponent_value: _,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
            Self::ParameterHasError {
                parameter_name: _,
                parameter_name_span: location_span,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
            Self::InvalidArgumentCount {
                function_name: _,
                function_name_span: location_span,
                expected_argument_count: _,
                actual_argument_count: _,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
            Self::ParameterMissingUnitAnnotation {
                param_expr_span: location_span,
                param_value_unit: _,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
            Self::ParameterUnitMismatch {
                param_expr_span: location_span,
                param_value_unit: _,
                param_unit_span: _,
                param_unit: _,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
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
            Self::MaxUnitDoesNotMatchMinUnit {
                max_unit: _,
                max_unit_span: location_span,
                min_unit: _,
                min_unit_span: _,
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
            Self::DiscreteLimitUnitMismatch {
                limit_unit: _,
                limit_span: _,
                value_unit: _,
                value_unit_span: location_span,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
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
            Self::StringCannotHaveNumberLimit {
                param_expr_span: _,
                param_value: _,
                limit_span: location_span,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
            Self::NumberCannotHaveStringLimit {
                param_expr_span: _,
                param_value: _,
                limit_span: location_span,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
            Self::UnitlessNumberCannotHaveLimitWithUnit {
                param_expr_span: _,
                param_value: _,
                limit_span: location_span,
                limit_unit: _,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
            Self::LimitUnitDoesNotMatchParameterUnit {
                param_unit: _,
                limit_span: location_span,
                limit_unit: _,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
            Self::Unsupported {
                relevant_span: location_span,
                feature_name: _,
                will_be_supported: _,
            } => Some(ErrorLocation::from_source_and_span(source, *location_span)),
        }
    }

    fn context(&self) -> Vec<ErrorContext> {
        match self {
            Self::BinaryEvalError {
                lhs_span: _,
                rhs_span: _,
                error,
            } => match error {
                BinaryEvalError::UnitMismatch {
                    lhs_unit: _,
                    rhs_unit: _,
                } => Vec::new(),
                BinaryEvalError::TypeMismatch {
                    lhs_type: _,
                    rhs_type: _,
                } => Vec::new(),
                BinaryEvalError::InvalidType { op: _, lhs_type: _ } => Vec::new(),
                BinaryEvalError::ExponentHasUnits { exponent_unit } => vec![ErrorContext::Note(
                    format!("exponent has units `{exponent_unit}`"),
                )],
                BinaryEvalError::ExponentIsInterval { exponent_interval } => {
                    vec![ErrorContext::Note(format!(
                        "exponent is <{} | {}>",
                        exponent_interval.min(),
                        exponent_interval.max()
                    ))]
                }
                BinaryEvalError::InvalidExponentType { exponent_type: _ } => Vec::new(),
            },
            Self::UnaryEvalError {
                expr_span: _,
                error,
            } => match error {
                UnaryEvalError::InvalidType {
                    op: _,
                    value_type: _,
                } => Vec::new(),
            },
            Self::HasExponentWithUnits {
                exponent_span: _,
                exponent_value,
            } => {
                vec![ErrorContext::Note(format!(
                    "exponent evaluated to {exponent_value}"
                ))]
            }
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

    fn context_with_source(&self, source: &str) -> Vec<(ErrorContext, Option<ErrorLocation>)> {
        match self {
            Self::BinaryEvalError {
                lhs_span,
                rhs_span: _,
                error,
            } => match error {
                BinaryEvalError::UnitMismatch {
                    lhs_unit,
                    rhs_unit: _,
                } => vec![(
                    ErrorContext::Note(format!("this expression has unit `{lhs_unit}`")),
                    Some(ErrorLocation::from_source_and_span(source, *lhs_span)),
                )],
                BinaryEvalError::TypeMismatch { lhs_type, rhs_type } => vec![(
                    ErrorContext::Note(format!("this expression has type `{lhs_type}`")),
                    Some(ErrorLocation::from_source_and_span(source, *lhs_span)),
                )],
                BinaryEvalError::InvalidType { op: _, lhs_type: _ } => Vec::new(),
                BinaryEvalError::ExponentHasUnits { exponent_unit: _ } => Vec::new(),
                BinaryEvalError::ExponentIsInterval {
                    exponent_interval: _,
                } => Vec::new(),
                BinaryEvalError::InvalidExponentType { exponent_type: _ } => Vec::new(),
            },
            Self::UnaryEvalError {
                expr_span: _,
                error,
            } => match error {
                UnaryEvalError::InvalidType {
                    op: _,
                    value_type: _,
                } => Vec::new(),
            },
            Self::HasExponentWithUnits {
                exponent_span: _,
                exponent_value: _,
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
                ErrorContext::Note(format!("expected unit was derived from this expression")),
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
