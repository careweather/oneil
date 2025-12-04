use crate::value::ValueError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalError {
    UndefinedBuiltinValue,
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
    Unsupported,
    NoNonEmptyValue,
}
