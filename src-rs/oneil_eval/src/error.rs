use crate::value::ValueError;

pub enum EvalError {
    ValueError(ValueError),
    HasIntervalExponent,
    HasExponentWithUnits,
    InvalidInterval,
    InvalidOperation,
    InvalidType,
    InvalidUnit,
}
