use crate::value::Unit;

/// The type of a value
#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    /// A boolean value type
    Boolean,
    /// A string value
    String,
    /// A number value type
    Number {
        /// The unit of the number value
        unit: Unit,
        /// The type of the number value
        number_type: NumberType,
    },
}

/// The type of a number value
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumberType {
    /// A scalar number value type
    Scalar,
    /// An interval number value type
    Interval,
}
