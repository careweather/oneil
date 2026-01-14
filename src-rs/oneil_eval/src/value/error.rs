use oneil_shared::span::Span;

use crate::{
    EvalError,
    error::ExpectedType,
    value::{DisplayUnit, Interval, ValueType},
};

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
    InvalidLhsType {
        expected_type: ExpectedType,
        lhs_type: ValueType,
    },
    InvalidRhsType {
        expected_type: ExpectedType,
        rhs_type: ValueType,
    },
    ExponentHasUnits {
        exponent_unit: DisplayUnit,
    },
    ExponentIsInterval {
        exponent_interval: Interval,
    },
}

impl BinaryEvalError {
    pub fn into_eval_error(self, lhs_span: Span, rhs_span: Span) -> EvalError {
        match self {
            Self::UnitMismatch { lhs_unit, rhs_unit } => EvalError::UnitMismatch {
                expected_unit: lhs_unit,
                expected_source_span: lhs_span,
                found_unit: rhs_unit,
                found_span: rhs_span,
            },
            Self::TypeMismatch { lhs_type, rhs_type } => EvalError::TypeMismatch {
                expected_type: lhs_type,
                expected_source_span: lhs_span,
                found_type: rhs_type,
                found_span: rhs_span,
            },
            Self::InvalidLhsType {
                expected_type,
                lhs_type,
            } => EvalError::InvalidType {
                expected_type,
                found_type: lhs_type,
                found_span: lhs_span,
            },
            Self::InvalidRhsType {
                expected_type,
                rhs_type,
            } => EvalError::InvalidType {
                expected_type,
                found_type: rhs_type,
                found_span: rhs_span,
            },
            Self::ExponentHasUnits { exponent_unit } => EvalError::ExponentHasUnits {
                exponent_span: rhs_span,
                exponent_unit,
            },
            Self::ExponentIsInterval { exponent_interval } => EvalError::ExponentIsInterval {
                exponent_interval,
                exponent_value_span: rhs_span,
            },
        }
    }

    pub fn expect_only_lhs_error(self, lhs_span: Span) -> EvalError {
        match self {
            Self::InvalidLhsType {
                expected_type,
                lhs_type,
            } => EvalError::InvalidType {
                expected_type,
                found_type: lhs_type,
                found_span: lhs_span,
            },
            Self::UnitMismatch { .. }
            | Self::TypeMismatch { .. }
            | Self::InvalidRhsType { .. }
            | Self::ExponentHasUnits { .. }
            | Self::ExponentIsInterval { .. } => {
                panic!("expected only lhs errors, but got {:?}", self)
            }
        }
    }

    pub fn expect_only_rhs_error(self, rhs_span: Span) -> EvalError {
        match self {
            Self::InvalidRhsType {
                expected_type,
                rhs_type,
            } => EvalError::InvalidType {
                expected_type,
                found_type: rhs_type,
                found_span: rhs_span,
            },
            Self::ExponentHasUnits { exponent_unit } => EvalError::ExponentHasUnits {
                exponent_span: rhs_span,
                exponent_unit,
            },
            Self::ExponentIsInterval { exponent_interval } => EvalError::ExponentIsInterval {
                exponent_interval,
                exponent_value_span: rhs_span,
            },
            Self::UnitMismatch { .. } | Self::TypeMismatch { .. } | Self::InvalidLhsType { .. } => {
                panic!("expected only rhs errors, but got {:?}", self)
            }
        }
    }

    pub fn expect_no_errors(self) -> EvalError {
        panic!("expected no errors, but got {:?}", self)
    }
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

impl UnaryEvalError {
    pub fn into_eval_error(self, value_span: Span) -> EvalError {
        match self {
            Self::InvalidType { op, value_type } => {
                let expected_type = match op {
                    UnaryOperation::Neg => ExpectedType::Number,
                    UnaryOperation::Not => ExpectedType::Boolean,
                };

                EvalError::InvalidType {
                    expected_type,
                    found_type: value_type,
                    found_span: value_span,
                }
            }
        }
    }

    pub fn expect_no_errors(self) -> EvalError {
        panic!("expected no errors, but got {:?}", self)
    }
}
