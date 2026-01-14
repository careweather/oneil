use std::{fmt, path::PathBuf};

use oneil_shared::{
    error::{AsOneilError, Context as ErrorContext, ErrorLocation},
    span::Span,
};

use crate::value::{DisplayUnit, Interval, NumberType, Value, ValueType};

/// An error that occurred during model evaluation.
///
/// This error type associates an evaluation error with the path to the model file
/// where the error occurred.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelError {
    /// The path to the model file where the error occurred.
    pub model_path: PathBuf,
    /// The evaluation error that occurred.
    pub error: EvalError,
}

/// Represents the expected type for type checking operations.
///
/// This enum is used to specify what type is expected in various type checking
/// contexts, such as function arguments or expression evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpectedType {
    /// A boolean value.
    Boolean,
    /// A string value.
    String,
    /// A unitless number (scalar or interval without units).
    Number,
    /// A number with a unit (measured number).
    MeasuredNumber,
    /// Either a unitless number or a number with a unit.
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

/// Represents the expected number of arguments for a function call.
///
/// This enum is used to specify argument count requirements when validating
/// function calls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpectedArgumentCount {
    /// Exactly the specified number of arguments is required.
    Exact(usize),
    /// At least the specified number of arguments is required.
    AtLeast(usize),
    /// At most the specified number of arguments is allowed.
    AtMost(usize),
    /// Between the minimum (inclusive) and maximum (inclusive) number of arguments is required.
    Between(usize, usize),
}

/// Errors that can occur during expression evaluation.
///
/// This enum represents all possible errors that can occur when evaluating
/// Oneil expressions, including type mismatches, unit errors, parameter validation
/// errors, and limit constraint violations.
#[derive(Debug, Clone, PartialEq)]
pub enum EvalError {
    /// A type mismatch error where the expected and found types do not match.
    ///
    /// This occurs when an expression is expected to have a specific type
    /// because of the type of a previous expression, but evaluates to a
    /// different type.
    TypeMismatch {
        /// The expected value type.
        expected_type: ValueType,
        /// The source span of the expression that caused the expected type.
        expected_source_span: Span,
        /// The actual value type that was found.
        found_type: ValueType,
        /// The source span of the expression that produced the wrong type.
        found_span: Span,
    },
    /// A unit mismatch error where the expected and found units do not match.
    ///
    /// This occurs when an expression is expected to have a specific unit
    /// because of the unit of a previous expression, but evaluates to a
    /// evaluates to a different unit.
    UnitMismatch {
        /// The expected unit.
        expected_unit: DisplayUnit,
        /// The source span of the expression that expected this unit.
        expected_source_span: Span,
        /// The actual unit that was found.
        found_unit: DisplayUnit,
        /// The source span of the expression that produced the wrong unit.
        found_span: Span,
    },
    /// An invalid type error where the found type does not match the expected type category.
    ///
    /// This occurs when an operation expects an expression to be a specific type category
    /// (e.g., number, boolean, string) but the expression evaluates to a different type.
    InvalidType {
        /// The expected type category.
        expected_type: ExpectedType,
        /// The actual value type that was found.
        found_type: ValueType,
        /// The source span of the expression that produced the wrong type.
        found_span: Span,
    },
    /// An invalid number type error where the found number type does not match the expected one.
    ///
    /// This occurs when an operation expects an expression to be a scalar or interval but
    /// the expression evaluates to the opposite number type.
    InvalidNumberType {
        /// The expected number type (scalar or interval).
        number_type: NumberType,
        /// The actual number type that was found.
        found_number_type: NumberType,
        /// The source span of the expression that produced the wrong number type.
        found_span: Span,
    },
    /// An error indicating that an exponent expression has units, which is not allowed.
    ///
    /// Exponents in power operations must be unitless numbers.
    ExponentHasUnits {
        /// The source span of the exponent expression.
        exponent_span: Span,
        /// The unit that was found on the exponent.
        exponent_unit: DisplayUnit,
    },
    /// An error indicating that an exponent expression evaluates to an interval, which is not allowed
    /// if the base has a unit.
    ///
    /// Exponents in power operations must be scalar values, not intervals.
    ExponentIsInterval {
        /// The interval value that the exponent evaluated to.
        exponent_interval: Interval,
        /// The source span of the exponent expression.
        exponent_value_span: Span,
    },
    /// An error indicating that a parameter has errors that prevent its evaluation.
    ///
    /// This occurs when a parameter is referenced but has errors that make it
    /// impossible to evaluate the current parameter.
    ///
    /// For error reporting, this error can typically be ignored. The main purpose of this error
    /// is error propagation, not error reporting.
    ParameterHasError {
        /// The name of the parameter that has errors.
        parameter_name: String,
        /// The source span of the parameter name.
        parameter_name_span: Span,
    },
    /// An error indicating that a function was called with an invalid number of arguments.
    InvalidArgumentCount {
        /// The name of the function that was called incorrectly.
        function_name: String,
        /// The source span of the function name.
        function_name_span: Span,
        /// The expected argument count specification.
        expected_argument_count: ExpectedArgumentCount,
        /// The actual number of arguments that were provided.
        actual_argument_count: usize,
    },
    /// An error indicating that a parameter value has a unit but the parameter is missing a unit annotation.
    ///
    /// This occurs when a parameter's value expression evaluates to a value with
    /// a unit, but the parameter definition does not include a unit annotation.
    ParameterMissingUnitAnnotation {
        /// The source span of the parameter expression.
        param_expr_span: Span,
        /// The unit that the parameter value has.
        param_value_unit: DisplayUnit,
    },
    /// An error indicating that a parameter value's unit does not match the parameter's declared unit.
    ///
    /// This occurs when a parameter's value expression evaluates to a value with
    /// a different unit than what is specified in the parameter's unit annotation.
    ParameterUnitMismatch {
        /// The source span of the parameter expression.
        param_expr_span: Span,
        /// The unit that the parameter value has.
        param_value_unit: DisplayUnit,
        /// The source span of the parameter's unit annotation.
        param_unit_span: Span,
        /// The unit that the parameter is declared to have.
        param_unit: DisplayUnit,
    },
    /// An error indicating that an unknown unit name was used.
    ///
    /// This occurs when a unit name is referenced that is not recognized by
    /// the unit system.
    UnknownUnit {
        /// The name of the unknown unit.
        unit_name: String,
        /// The source span of the unit name.
        unit_name_span: Span,
    },
    /// An error indicating that a piecewise expression condition does not evaluate to a boolean.
    InvalidIfExpressionType {
        /// The source span of the conditional expression.
        expr_span: Span,
        /// The value that the expression evaluated to.
        found_value: Value,
    },
    /// An error indicating that multiple piecewise branches match for a parameter.
    MultiplePiecewiseBranchesMatch {
        /// The name of the parameter being evaluated.
        param_ident: String,
        /// The source span of the parameter identifier.
        param_ident_span: Span,
        /// The source spans of all matching branch conditions.
        matching_branche_spans: Vec<Span>,
    },
    /// An error indicating that no piecewise branch matches for a parameter.
    NoPiecewiseBranchMatch {
        /// The name of the parameter being evaluated.
        param_ident: String,
        /// The source span of the parameter identifier.
        param_ident_span: Span,
    },
    /// An error indicating that a boolean value cannot have a unit annotation.
    BooleanCannotHaveUnit {
        /// The source span of the boolean expression.
        expr_span: Span,
        /// The source span of the unit annotation.
        unit_span: Span,
    },
    /// An error indicating that a string value cannot have a unit annotation.
    StringCannotHaveUnit {
        /// The source span of the string expression.
        expr_span: Span,
        /// The source span of the unit annotation.
        unit_span: Span,
    },
    /// An error indicating that a continuous limit minimum expression does not evaluate to a number.
    InvalidContinuousLimitMinType {
        /// The source span of the minimum limit expression.
        expr_span: Span,
        /// The value that the expression evaluated to.
        found_value: Value,
    },
    /// An error indicating that a continuous limit maximum expression does not evaluate to a number.
    InvalidContinuousLimitMaxType {
        /// The source span of the maximum limit expression.
        expr_span: Span,
        /// The value that the expression evaluated to.
        found_value: Value,
    },
    /// An error indicating that a continuous limit's max unit does not match its min unit.
    MaxUnitDoesNotMatchMinUnit {
        /// The unit of the maximum limit value.
        max_unit: DisplayUnit,
        /// The source span of the maximum limit expression.
        max_unit_span: Span,
        /// The unit of the minimum limit value.
        min_unit: DisplayUnit,
        /// The source span of the minimum limit expression.
        min_unit_span: Span,
    },
    /// An error indicating that a boolean value cannot be used as a discrete limit value.
    BooleanCannotBeDiscreteLimitValue {
        /// The source span of the boolean expression.
        expr_span: Span,
    },
    /// An error indicating that a duplicate string value appears in a discrete limit.
    DuplicateStringLimit {
        /// The source span of the duplicate string expression.
        expr_span: Span,
        /// The source span of the original string expression.
        original_expr_span: Span,
        /// The string value that was duplicated.
        string_value: String,
    },
    /// An error indicating that a discrete limit value was expected to be a string but was not.
    ExpectedStringLimit {
        /// The source span of the invalid limit expression.
        expr_span: Span,
        /// The value that the expression evaluated to.
        found_value: Value,
    },
    /// An error indicating that a discrete limit value was expected to be a number but was not.
    ExpectedNumberLimit {
        /// The source span of the invalid limit expression.
        expr_span: Span,
        /// The value that the expression evaluated to.
        found_value: Value,
    },
    /// An error indicating that a discrete limit value's unit does not match the expected unit.
    DiscreteLimitUnitMismatch {
        /// The expected unit for the limit values.
        limit_unit: DisplayUnit,
        /// The source span of the source of the expected unit.
        limit_span: Span,
        /// The unit that the limit value has.
        value_unit: DisplayUnit,
        /// The source span of the limit value expression.
        value_unit_span: Span,
    },
    /// An error indicating that a parameter value is below the default parameter limits.
    ///
    /// This occurs when a parameter value is negative, which violates
    /// the default limits for number parameters (0, inf).
    ParameterValueBelowDefaultLimits {
        /// The source span of the parameter expression.
        param_expr_span: Span,
        /// The parameter value that violates the limits.
        param_value: Value,
    },
    /// An error indicating that a parameter value is below its continuous limit minimum.
    ParameterValueBelowContinuousLimits {
        /// The source span of the parameter expression.
        param_expr_span: Span,
        /// The parameter value that violates the limit.
        param_value: Value,
        /// The source span of the minimum limit expression.
        min_expr_span: Span,
        /// The minimum limit value.
        min_value: Value,
    },
    /// An error indicating that a parameter value is above its continuous limit maximum.
    ParameterValueAboveContinuousLimits {
        /// The source span of the parameter expression.
        param_expr_span: Span,
        /// The parameter value that violates the limit.
        param_value: Value,
        /// The source span of the maximum limit expression.
        max_expr_span: Span,
        /// The maximum limit value.
        max_value: Value,
    },
    /// An error indicating that a parameter value is not in its discrete limit list.
    ParameterValueNotInDiscreteLimits {
        /// The source span of the parameter expression.
        param_expr_span: Span,
        /// The parameter value that is not in the limit list.
        param_value: Value,
        /// The source span of the discrete limit definition.
        limit_expr_span: Span,
        /// The list of allowed limit values.
        limit_values: Vec<Value>,
    },
    /// An error indicating that a boolean parameter cannot have a limit.
    BooleanCannotHaveALimit {
        /// The source span of the boolean parameter expression.
        expr_span: Span,
        /// The source span of the limit definition.
        limit_span: Span,
    },
    /// An error indicating that a string parameter cannot have a number limit.
    StringCannotHaveNumberLimit {
        /// The source span of the string parameter expression.
        param_expr_span: Span,
        /// The parameter value.
        param_value: Value,
        /// The source span of the limit definition.
        limit_span: Span,
    },
    /// An error indicating that a number parameter cannot have a string limit.
    NumberCannotHaveStringLimit {
        /// The source span of the number parameter expression.
        param_expr_span: Span,
        /// The parameter value.
        param_value: Value,
        /// The source span of the limit definition.
        limit_span: Span,
    },
    /// An error indicating that a unitless number parameter cannot have a limit with units.
    UnitlessNumberCannotHaveLimitWithUnit {
        /// The source span of the parameter expression.
        param_expr_span: Span,
        /// The parameter value.
        param_value: Value,
        /// The source span of the limit definition.
        limit_span: Span,
        /// The unit that the limit has.
        limit_unit: DisplayUnit,
    },
    /// An error indicating that a limit's unit does not match the parameter's unit.
    LimitUnitDoesNotMatchParameterUnit {
        /// The unit that the parameter is declared to have.
        param_unit: DisplayUnit,
        /// The source span of the limit definition.
        limit_span: Span,
        /// The unit that the limit has.
        limit_unit: DisplayUnit,
    },
    /// An error indicating that an unsupported feature was used.
    ///
    /// This occurs when attempting to use a language feature that is not yet
    /// implemented or is not supported.
    Unsupported {
        /// The source span of the unsupported feature usage.
        relevant_span: Span,
        /// The name of the unsupported feature, if known.
        feature_name: Option<String>,
        /// Whether this feature is planned to be supported in the future.
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
