//! Errors that can occur when evaluating binary or unary operations on values.
//!
//! These error types are used during expression evaluation. Conversion to
//! evaluator-level errors (`EvalError`) is done by the `oneil_eval` crate.

use crate::{DisplayUnit, Interval, ValueType};

/// Represents the expected type for type checking operations in value-level errors.
///
/// This mirrors the evaluator's `ExpectedType` for use in binary/unary eval errors.
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

/// Errors that can occur when evaluating a binary operation.
///
/// Note that all `ValueType`s are boxed to decrease the size
/// of the error enum.
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryEvalError {
    /// Unit mismatch between operands.
    UnitMismatch {
        /// Unit of the left-hand side.
        lhs_unit: DisplayUnit,
        /// Unit of the right-hand side.
        rhs_unit: DisplayUnit,
    },
    /// Type mismatch between operands.
    TypeMismatch {
        /// Type of the left-hand side.
        lhs_type: Box<ValueType>,
        /// Type of the right-hand side.
        rhs_type: Box<ValueType>,
    },
    /// Left-hand side has an invalid type.
    InvalidLhsType {
        /// Type that was expected for the left-hand side.
        expected_type: ExpectedType,
        /// Actual type of the left-hand side.
        lhs_type: Box<ValueType>,
    },
    /// Right-hand side has an invalid type.
    InvalidRhsType {
        /// Type that was expected for the right-hand side.
        expected_type: ExpectedType,
        /// Actual type of the right-hand side.
        rhs_type: Box<ValueType>,
    },
    /// Exponent has units (not allowed).
    ExponentHasUnits {
        /// Unit of the exponent (must be unitless).
        exponent_unit: DisplayUnit,
    },
    /// Exponent is an interval (not allowed when base has unit).
    ExponentIsInterval {
        /// Interval used as exponent (must be scalar when base has unit).
        exponent_interval: Interval,
    },
}

/// Unary operations that can fail with a type error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperation {
    /// Negation.
    Neg,
    /// Logical not.
    Not,
}

/// Errors that can occur when evaluating a unary operation.
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryEvalError {
    /// The value has an invalid type for the operation.
    InvalidType {
        /// The unary operation that was applied.
        op: UnaryOperation,
        /// Actual type of the value (invalid for this operation).
        value_type: Box<ValueType>,
    },
}

/// Errors that can occur when converting a value to a specific unit.
#[derive(Debug, Clone, PartialEq)]
pub enum UnitConversionError {
    /// Unit mismatch between the value unit and the requested target unit.
    UnitMismatch {
        /// Unit of the value being converted.
        value_unit: DisplayUnit,
        /// Unit requested by the caller.
        target_unit: DisplayUnit,
    },
    /// Value type is not convertible to the target unit.
    InvalidType {
        /// Value type of the value that could not be converted.
        value_type: Box<ValueType>,
        /// Requested target unit for the conversion.
        target_unit: Box<DisplayUnit>,
    },
}
