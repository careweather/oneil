use crate::value::ValueError;

pub enum EvalError {
    ValueError(ValueError),
    ParameterUnitMismatch,
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
}
