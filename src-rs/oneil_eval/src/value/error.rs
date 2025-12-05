#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueError {
    InvalidUnit,
    HasExponentWithUnits,
    HasIntervalExponent,
    InvalidOperation,
    InvalidType,
}
