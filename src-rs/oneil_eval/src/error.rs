#![expect(missing_docs, reason = "this enum will be reworked in the next task")]

use std::path::PathBuf;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalError {
    InvalidUnit,
    HasExponentWithUnits,
    HasIntervalExponent,
    InvalidOperation,
    InvalidType,
    ParameterHasError,
    UndefinedBuiltinValue,
    InvalidArgumentCount,
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
    Unsupported,
    NoNonEmptyValue,
    ModelNotFound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelError {
    pub model_path: PathBuf,
    pub error: EvalError,
}
