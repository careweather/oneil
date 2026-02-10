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
        lhs_unit: DisplayUnit,
        rhs_unit: DisplayUnit,
    },
    /// Type mismatch between operands.
    TypeMismatch {
        lhs_type: Box<ValueType>,
        rhs_type: Box<ValueType>,
    },
    /// Left-hand side has an invalid type.
    InvalidLhsType {
        expected_type: ExpectedType,
        lhs_type: Box<ValueType>,
    },
    /// Right-hand side has an invalid type.
    InvalidRhsType {
        expected_type: ExpectedType,
        rhs_type: Box<ValueType>,
    },
    /// Exponent has units (not allowed).
    ExponentHasUnits { exponent_unit: DisplayUnit },
    /// Exponent is an interval (not allowed when base has unit).
    ExponentIsInterval { exponent_interval: Interval },
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
        op: UnaryOperation,
        value_type: Box<ValueType>,
    },
}
