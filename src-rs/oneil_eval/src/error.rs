use crate::value::ValueError;

#[derive(Debug, Clone, PartialEq)]
pub enum EvalError {
    ValueError(ValueError),
    InvalidArgumentCount,
    ParameterUnitMismatch,
    InvalidType,
    InvalidUnit,
    UnknownUnit,
    InvalidIfExpressionType,
    MultiplePiecewiseBranchesMatch,
    NoPiecewiseBranchMatch,
    BooleanCannotHaveUnit,
    StringCannotHaveUnit,
    InvalidContinuousLimitMinType,
    InvalidContinuousLimitMaxType,
    LimitCannotBeBoolean,
    DuplicateStringLimit,
    ExpectedStringLimit,
    ExpectedNumberLimit,
    DiscreteLimitUnitMismatch,
    ParameterValueOutsideLimits,
    ParameterUnitDoesNotMatchLimit,
    NoNonEmptyValue,
}
