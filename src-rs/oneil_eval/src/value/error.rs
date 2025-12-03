#[derive(Debug, Clone, PartialEq)]
pub enum ValueError {
    InvalidUnit,
    HasExponentWithUnits,
    HasIntervalExponent,
    InvalidOperation,
    InvalidType,
}
