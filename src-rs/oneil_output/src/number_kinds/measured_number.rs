//! Measured number (number with unit).

use std::ops;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{Unit, error::BinaryEvalError};

use super::{NormalizedNumber, Number};

// TODO: document how this guarantees that the number is
//       correctly constructed

/// A number with a unit
///
/// The only way to construct a `MeasuredNumber` is to use `from_number_and_unit`,
/// which performs adjustment based on the magnitude/dB of the unit.
#[derive(Debug, Clone)]
pub struct MeasuredNumber {
    /// The value of the measured number.
    ///
    /// Note that this is stored in a "normalized" form,
    /// where the magnitude/dB of the unit is already applied.
    ///
    /// See `from_number_and_unit` for more details.
    normalized_value: NormalizedNumber,
    /// The unit of the measured number.
    unit: Unit,
}

impl MeasuredNumber {
    /// Converts a number and a unit to a measured number.
    ///
    /// Note that this performs adjustment based on the magnitude/dB of the unit.
    ///
    /// A measured number stores the number in a "normalized" form,
    /// where the magnitude/dB of the unit is already applied.
    ///
    /// For example, `value = 1` and `unit = km` will result in `1000 m`.
    #[must_use]
    pub fn from_number_and_unit(value: Number, unit: Unit) -> Self {
        let normalized_value = NormalizedNumber::from_number_and_unit(value, &unit);
        Self {
            normalized_value,
            unit,
        }
    }

    /// Converts a measured number to a number and a unit.
    ///
    /// This is the inverse of `from_number_and_unit`. See that function
    /// for a description of the conversion.
    #[must_use]
    pub fn into_number_and_unit(self) -> (Number, Unit) {
        let Self {
            normalized_value,
            unit,
        } = self;

        let value = normalized_value.into_number_using_unit(&unit);

        (value, unit)
    }

    /// Converts a measured number to a number based on the given unit.
    ///
    /// This performs adjustment based on the magnitude/dB of the unit.
    ///
    /// # Panics
    ///
    /// This panics if the new unit is not dimensionally equivalent
    /// to the old unit.
    #[must_use]
    pub fn into_number_using_unit(self, unit: &Unit) -> Number {
        debug_assert!(
            self.unit.dimensionally_eq(unit),
            "old unit {} is not dimensionally equivalent to new unit {}\nold unit dimensions: {:?}\nnew unit dimensions: {:?}",
            self.unit.display_unit,
            unit.display_unit,
            self.unit.dimension_map,
            unit.dimension_map,
        );

        self.normalized_value.into_number_using_unit(unit)
    }

    /// Updates the unit of the measured number.
    ///
    ///
    /// # Panics
    ///
    /// This panics if the new unit is not dimensionally equivalent
    /// to the old unit.
    #[must_use]
    pub fn with_unit(self, unit: Unit) -> Self {
        debug_assert!(
            self.unit.dimensionally_eq(&unit),
            "old unit {} is not dimensionally equivalent to new unit {}\nold unit dimensions: {:?}\nnew unit dimensions: {:?}",
            self.unit.display_unit,
            unit.display_unit,
            self.unit.dimension_map,
            unit.dimension_map,
        );

        Self { unit, ..self }
    }

    /// Returns the normalized value of the measured number.
    ///
    /// The normalized value can be compared with other normalized
    /// values, regardless of unit magnitude.
    ///
    /// For example, `1000 m` and `1 km` are normalized to the
    /// same value.
    #[must_use]
    pub const fn normalized_value(&self) -> &NormalizedNumber {
        &self.normalized_value
    }

    /// Returns the unit of the measured number.
    #[must_use]
    pub const fn unit(&self) -> &Unit {
        &self.unit
    }

    /// Returns whether this measured number is dimensionless.
    #[must_use]
    pub fn is_dimensionless(&self) -> bool {
        self.unit.is_dimensionless()
    }

    /// Returns whether this measured number is effectively unitless.
    ///
    /// A measured number is effectively unitless when its unit has magnitude 1,
    /// is not a dB unit, and is dimensionless.
    #[must_use]
    pub fn is_effectively_unitless(&self) -> bool {
        self.unit.is_effectively_unitless()
    }

    /// Checks if two measured numbers are equal.
    ///
    /// # Errors
    ///
    /// Returns `Err(BinaryEvalError::UnitMismatch)` if the dimensions don't match.
    pub fn checked_eq(&self, rhs: &Self) -> Result<bool, BinaryEvalError> {
        self.check_units(rhs)?;
        Ok(self.normalized_value == rhs.normalized_value)
    }

    /// Checks if `self` is strictly less than `rhs`.
    ///
    /// # Errors
    ///
    /// Returns `Err(BinaryEvalError::UnitMismatch)` if the units don't match.
    pub fn checked_lt(&self, rhs: &Self) -> Result<bool, BinaryEvalError> {
        self.check_units(rhs)?;
        Ok(self.normalized_value.lt(&rhs.normalized_value))
    }

    /// Checks if `self` is strictly greater than `rhs`.
    ///
    /// # Errors
    ///
    /// Returns `Err(BinaryEvalError::UnitMismatch)` if the units don't match.
    pub fn checked_gt(&self, rhs: &Self) -> Result<bool, BinaryEvalError> {
        self.check_units(rhs)?;
        Ok(self.normalized_value.gt(&rhs.normalized_value))
    }

    /// Checks if `self` is less than or equal to `rhs`.
    ///
    /// # Errors
    ///
    /// Returns `Err(BinaryEvalError::UnitMismatch)` if the units don't match.
    pub fn checked_lte(&self, rhs: &Self) -> Result<bool, BinaryEvalError> {
        self.check_units(rhs)?;
        Ok(self.normalized_value.lte(&rhs.normalized_value))
    }

    /// Checks if `self` is greater than or equal to `rhs`.
    ///
    /// # Errors
    ///
    /// Returns `Err(BinaryEvalError::UnitMismatch)` if the units don't match.
    pub fn checked_gte(&self, rhs: &Self) -> Result<bool, BinaryEvalError> {
        self.check_units(rhs)?;
        Ok(self.normalized_value.gte(&rhs.normalized_value))
    }

    /// Adds two dimensional numbers.
    ///
    /// # Errors
    ///
    /// Returns `Err(ValueError::InvalidUnit)` if the dimensions don't match.
    pub fn checked_add(self, rhs: &Self) -> Result<Self, BinaryEvalError> {
        self.check_units(rhs)?;
        Ok(Self {
            normalized_value: self.normalized_value + rhs.normalized_value,
            unit: self.unit,
        })
    }

    /// Subtracts two dimensional numbers.
    ///
    /// # Errors
    ///
    /// Returns `Err(ValueError::InvalidUnit)` if the dimensions don't match.
    pub fn checked_sub(self, rhs: &Self) -> Result<Self, BinaryEvalError> {
        self.check_units(rhs)?;
        Ok(Self {
            normalized_value: self.normalized_value - rhs.normalized_value,
            unit: self.unit,
        })
    }

    /// Subtracts two dimensional numbers. This does not apply the
    /// standard rules of interval arithmetic. Instead, it subtracts the minimum
    /// from the minimum and the maximum from the maximum.
    ///
    /// # Errors
    ///
    /// Returns `Err(ValueError::InvalidUnit)` if the dimensions don't match.
    pub fn checked_escaped_sub(self, rhs: &Self) -> Result<Self, BinaryEvalError> {
        self.check_units(rhs)?;
        Ok(Self {
            normalized_value: self.normalized_value.escaped_sub(rhs.normalized_value),
            unit: self.unit,
        })
    }

    /// Multiplies two dimensional numbers.
    ///
    /// # Errors
    ///
    /// This function never returns an error.
    pub fn checked_mul(self, rhs: Self) -> Result<Self, BinaryEvalError> {
        Ok(Self {
            normalized_value: self.normalized_value * rhs.normalized_value,
            unit: self.unit * rhs.unit,
        })
    }

    /// Divides two dimensional numbers.
    ///
    /// # Errors
    ///
    /// Returns `Err(ValueError::InvalidUnit)` if the dimensions don't match.
    pub fn checked_div(self, rhs: Self) -> Result<Self, BinaryEvalError> {
        Ok(Self {
            normalized_value: self.normalized_value / rhs.normalized_value,
            unit: self.unit / rhs.unit,
        })
    }

    /// Divides two dimensional numbers. This does not apply the
    /// standard rules of interval arithmetic. Instead, it divides the minimum
    /// by the minimum and the maximum by the maximum.
    ///
    /// # Errors
    ///
    /// Returns `Err(ValueError::InvalidUnit)` if the dimensions don't match.
    pub fn checked_escaped_div(self, rhs: Self) -> Result<Self, BinaryEvalError> {
        Ok(Self {
            normalized_value: self.normalized_value.escaped_div(rhs.normalized_value),
            unit: self.unit / rhs.unit,
        })
    }

    /// Computes the remainder of two dimensional numbers.
    ///
    /// # Errors
    ///
    /// Returns `Err(ValueError::InvalidUnit)` if the dimensions don't match.
    pub fn checked_rem(self, rhs: &Self) -> Result<Self, BinaryEvalError> {
        self.check_units(rhs)?;
        Ok(Self {
            normalized_value: self.normalized_value % rhs.normalized_value,
            unit: self.unit,
        })
    }

    /// Raises a dimensional number to the power of another.
    ///
    /// # Errors
    ///
    /// Returns `Err(ValueError::InvalidUnit)` if the dimensions don't match.
    pub fn checked_pow(self, exponent: &Number) -> Result<Self, BinaryEvalError> {
        match exponent {
            Number::Scalar(exponent_value) => Ok(Self {
                normalized_value: self.normalized_value.pow(Number::Scalar(*exponent_value)),
                unit: self.unit.pow(*exponent_value),
            }),
            Number::Interval(exponent_interval) => Err(BinaryEvalError::ExponentIsInterval {
                exponent_interval: *exponent_interval,
            }),
        }
    }

    /// Returns the tightest enclosing interval of two dimensional numbers.
    ///
    /// # Errors
    ///
    /// Returns `Err(ValueError::InvalidUnit)` if the dimensions don't match.
    pub fn checked_min_max(self, rhs: &Self) -> Result<Self, BinaryEvalError> {
        self.check_units(rhs)?;
        Ok(Self {
            normalized_value: self
                .normalized_value
                .tightest_enclosing_interval(rhs.normalized_value),
            unit: self.unit,
        })
    }

    /// Returns the intersection of two dimensional numbers.
    ///
    /// # Errors
    ///
    /// Returns `Err(ValueError::InvalidUnit)` if the dimensions don't match.
    pub fn checked_intersection(self, rhs: &Self) -> Result<Self, BinaryEvalError> {
        self.check_units(rhs)?;
        Ok(Self {
            normalized_value: self.normalized_value.intersection(rhs.normalized_value),
            unit: self.unit,
        })
    }

    /// Negates a number value.
    #[must_use]
    pub fn checked_neg(self) -> Self {
        Self {
            normalized_value: -self.normalized_value,
            unit: self.unit,
        }
    }

    /// Returns the minimum value of the measured number.
    #[must_use]
    pub fn min(&self) -> Self {
        Self {
            normalized_value: NormalizedNumber::from_normalized_value(Number::Scalar(
                self.normalized_value.min(),
            )),
            unit: self.unit.clone(),
        }
    }

    /// Returns the maximum value of the measured number.
    #[must_use]
    pub fn max(&self) -> Self {
        Self {
            normalized_value: NormalizedNumber::from_normalized_value(Number::Scalar(
                self.normalized_value.max(),
            )),
            unit: self.unit.clone(),
        }
    }

    /// Returns the square root of the measured number.
    #[must_use]
    pub fn sqrt(self) -> Self {
        Self {
            normalized_value: self.normalized_value.sqrt(),
            unit: self.unit.pow(0.5),
        }
    }

    /// Returns the natural logarithm of the measured number.
    #[must_use]
    pub fn ln(self) -> Self {
        Self {
            normalized_value: self.normalized_value.ln(),
            unit: self.unit,
        }
    }

    /// Returns the base 10 logarithm of the measured number.
    #[must_use]
    pub fn log10(self) -> Self {
        Self {
            normalized_value: self.normalized_value.log10(),
            unit: self.unit,
        }
    }

    /// Returns the base 2 logarithm of the measured number.
    #[must_use]
    pub fn log2(self) -> Self {
        Self {
            normalized_value: self.normalized_value.log2(),
            unit: self.unit,
        }
    }

    /// Returns the absolute value of the measured number.
    #[must_use]
    pub fn abs(self) -> Self {
        Self {
            normalized_value: self.normalized_value.abs(),
            unit: self.unit,
        }
    }

    /// Returns the measured number rounded down to the nearest integer.
    #[must_use]
    pub fn floor(self) -> Self {
        Self {
            normalized_value: self.normalized_value.floor(),
            unit: self.unit,
        }
    }

    /// Returns the measured number rounded up to the nearest integer.
    #[must_use]
    pub fn ceiling(self) -> Self {
        Self {
            normalized_value: self.normalized_value.ceiling(),
            unit: self.unit,
        }
    }

    fn check_units(&self, rhs: &Self) -> Result<(), BinaryEvalError> {
        if self.unit.dimensionally_eq(&rhs.unit) {
            Ok(())
        } else {
            Err(BinaryEvalError::UnitMismatch {
                lhs_unit: self.unit.display_unit.clone(),
                rhs_unit: rhs.unit.display_unit.clone(),
            })
        }
    }
}

impl PartialEq for MeasuredNumber {
    /// Compares two measured numbers for equality.
    ///
    /// This treats units as equal if they have the same dimensions.
    fn eq(&self, other: &Self) -> bool {
        self.check_units(other).is_ok() && self.normalized_value == other.normalized_value
    }
}

impl ops::Mul<Number> for MeasuredNumber {
    type Output = Self;

    /// Multiply a measured number by a number.
    ///
    /// The `Number` is implicitly coerced to a
    /// measured number with unit `1`.
    fn mul(self, rhs: Number) -> Self::Output {
        Self {
            normalized_value: self.normalized_value * rhs,
            unit: self.unit,
        }
    }
}

impl ops::Mul<MeasuredNumber> for Number {
    type Output = MeasuredNumber;

    /// Multiply a number by a measured number.
    ///
    /// The `Number` is implicitly coerced to a
    /// measured number with unit `1`.
    fn mul(self, rhs: MeasuredNumber) -> Self::Output {
        MeasuredNumber {
            normalized_value: self * rhs.normalized_value,
            unit: rhs.unit,
        }
    }
}

impl ops::Div<Number> for MeasuredNumber {
    type Output = Self;

    /// Divide a measured number by a number.
    ///
    /// The `Number` is implicitly coerced to a dimensionless
    /// measured number.
    fn div(self, rhs: Number) -> Self::Output {
        Self {
            normalized_value: self.normalized_value / rhs,
            unit: self.unit,
        }
    }
}

impl ops::Div<MeasuredNumber> for Number {
    type Output = MeasuredNumber;

    /// Divide a number by a measured number.
    ///
    /// The `Number` is implicitly coerced to a dimensionless
    /// measured number.
    fn div(self, rhs: MeasuredNumber) -> Self::Output {
        MeasuredNumber {
            normalized_value: self / rhs.normalized_value,
            unit: Unit::one() / rhs.unit,
        }
    }
}

/// A measured number in a human-readable format.
#[derive(Serialize, Deserialize)]
struct HumanReadableMeasuredNumber {
    /// The value of the measured number.
    value: Number,
    /// The unit of the measured number.
    unit: Unit,
}

impl Serialize for MeasuredNumber {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let (value, unit) = self.clone().into_number_and_unit();
        let human_readable = HumanReadableMeasuredNumber { value, unit };
        human_readable.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MeasuredNumber {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let human_readable = HumanReadableMeasuredNumber::deserialize(deserializer)?;
        let HumanReadableMeasuredNumber { value, unit } = human_readable;
        Ok(Self::from_number_and_unit(value, unit))
    }
}

#[cfg(test)]
mod serde_tests {
    use std::collections::BTreeMap;

    use crate::{Dimension, DimensionMap, DisplayUnit, Interval, Value};

    use super::*;

    fn sample_unit() -> Unit {
        Unit {
            dimension_map: DimensionMap::new(BTreeMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Distance, 2.0),
                (Dimension::Time, -3.0),
            ])),
            magnitude: 1.0,
            is_db: false,
            display_unit: DisplayUnit::Unit {
                name: "W".to_string(),
                exponent: 1.0,
            },
        }
    }

    #[test]
    fn measured_scalar_round_trip() {
        let m = MeasuredNumber::from_number_and_unit(Number::Scalar(100.0), sample_unit());
        let json = serde_json::to_string(&m).expect("serialize");
        let back: MeasuredNumber = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, m);
    }

    #[test]
    fn measured_interval_round_trip() {
        let m = MeasuredNumber::from_number_and_unit(
            Number::Interval(Interval::new(1.0, 2.0)),
            sample_unit(),
        );
        let json = serde_json::to_string(&m).expect("serialize");
        let back: MeasuredNumber = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, m);
    }

    #[test]
    fn measured_json_matches_value_measured_json() {
        let m = MeasuredNumber::from_number_and_unit(Number::Scalar(2.5), Unit::one());
        let as_value = Value::MeasuredNumber(m.clone());
        assert_eq!(
            serde_json::to_string(&m).expect("serialize measured"),
            serde_json::to_string(&as_value).expect("serialize value"),
        );
    }
}
