use crate::value::Unit;

/// The type of a value
#[derive(Debug, Clone)]
pub enum ValueType {
    /// A boolean value type
    Boolean,
    /// A string value
    String,
    /// A number value type
    Number {
        /// The type of the number value
        number_type: NumberType,
    },
    /// A measured number value type
    MeasuredNumber {
        /// The unit of the number value
        unit: Unit,
        /// The type of the number value
        number_type: NumberType,
    },
}

impl PartialEq for ValueType {
    /// Compares two value types for equality.
    ///
    /// This treats units as equal if they have the same dimensions.
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Boolean, Self::Boolean) | (Self::String, Self::String) => true,
            (
                Self::Number {
                    number_type: lhs_number_type,
                },
                Self::Number {
                    number_type: rhs_number_type,
                },
            ) => lhs_number_type == rhs_number_type,
            (
                Self::MeasuredNumber {
                    unit: lhs_unit,
                    number_type: lhs_number_type,
                },
                Self::MeasuredNumber {
                    unit: rhs_unit,
                    number_type: rhs_number_type,
                },
            ) => lhs_number_type == rhs_number_type && lhs_unit.dimensionally_eq(rhs_unit),
            (_, _) => false,
        }
    }
}

/// The type of a number value
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumberType {
    /// A scalar number value type
    Scalar,
    /// An interval number value type
    Interval,
}
