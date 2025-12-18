use std::{collections::HashMap, ops};

use crate::value::util::is_close;

/// The dimension of a base unit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dimension {
    /// Base unit is 'kilogram'
    Mass,
    /// Base unit is 'meter'
    Distance,
    /// Base unit is 'second'
    Time,
    /// Base unit is 'kelvin'
    Temperature,
    /// Base unit is 'ampere'
    Current,
    /// Base unit is 'bit'
    Information,
    /// Base unit is 'USD'
    Currency,
    /// Base unit is 'mole'
    Substance,
    /// Base unit is 'candela'
    LuminousIntensity,
}

/// Represents a unit in Oneil.
///
/// A unit is a collection of dimensions and their exponents.
///
/// For example, "m/s" is represented as `Unit(HashMap::from([(Dimension::Distance, 1.0), (Dimension::Time, -1.0)]))`.
#[derive(Debug, Clone)]
pub struct Unit(HashMap<Dimension, f64>);

impl Unit {
    /// Creates a new unit from a map of dimensions and their exponents.
    #[must_use]
    pub const fn new(units: HashMap<Dimension, f64>) -> Self {
        Self(units)
    }

    /// Creates a unitless unit, which is a unit with no dimensions.
    #[must_use]
    pub fn unitless() -> Self {
        Self(HashMap::new())
    }

    /// Checks if the unit is unitless (has no dimensions).
    #[must_use]
    pub fn is_unitless(&self) -> bool {
        self.0.is_empty()
    }

    /// Raises the unit to the power of the given exponent.
    #[must_use]
    pub fn pow(self, exponent: f64) -> Self {
        Self(
            self.0
                .into_iter()
                .map(|(key, value)| (key, value * exponent))
                .collect(),
        )
    }
}

impl PartialEq for Unit {
    /// Checks if two units are equal
    ///
    /// Note that this is a fuzzy equality check, and
    /// that the units are considered equal if their
    /// dimensions and exponents are close, as determined
    /// by the `is_close` function.
    fn eq(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }

        self.0.iter().all(|(dimension, value)| {
            let other_value = other.0.get(dimension);
            other_value.is_some_and(|other_value| is_close(*other_value, *value))
        })
    }
}

impl ops::Mul for Unit {
    type Output = Self;

    /// Multiplies two units together
    ///
    /// For example, `(m/s) * (g) = (g*m/s)`
    fn mul(self, rhs: Self) -> Self::Output {
        let mut result = self.0;

        for (key, value) in rhs.0 {
            result
                .entry(key)
                .and_modify(|v| *v += value)
                .or_insert(value);
        }

        Self(
            result
                .into_iter()
                .filter(|(_, value)| *value != 0.0)
                .collect(),
        )
    }
}

impl ops::Div for Unit {
    type Output = Self;

    /// Divides two units
    ///
    /// For example, `(g*m/s) / (g) = (m/s)`
    fn div(self, rhs: Self) -> Self::Output {
        let mut result = self.0;

        for (key, value) in rhs.0 {
            #[expect(
                clippy::suspicious_arithmetic_impl,
                reason = "division is defined as subtraction of the exponent"
            )]
            result
                .entry(key)
                .and_modify(|v| *v -= value)
                .or_insert(-value);
        }

        Self(
            result
                .into_iter()
                .filter(|(_, value)| *value != 0.0)
                .collect(),
        )
    }
}

/// Represents a sized unit in Oneil.
///
/// A sized unit is a unit with a magnitude. This is useful for
/// representing units such as `kg = 1000 g` or `ms = 0.001 s`.
#[derive(Debug, Clone, PartialEq)]
pub struct SizedUnit {
    /// The magnitude of the sized unit.
    pub magnitude: f64,
    /// The unit of the sized unit.
    pub unit: Unit,
    /// Whether the sized unit is a dB unit.
    pub is_db: bool,
}

impl SizedUnit {
    /// Creates a new unitless sized unit.
    #[must_use]
    pub fn unitless() -> Self {
        Self {
            magnitude: 1.0,
            unit: Unit::unitless(),
            is_db: false,
        }
    }

    /// Raises the sized unit to the power of the given exponent.
    #[must_use]
    pub fn pow(self, exponent: f64) -> Self {
        Self {
            magnitude: self.magnitude.powf(exponent),
            unit: self.unit.pow(exponent),
            is_db: self.is_db,
        }
    }

    /// Sets the is_db flag of the sized unit.
    #[must_use]
    pub fn set_is_db(self, is_db: bool) -> Self {
        Self { is_db, ..self }
    }
}

impl ops::Mul for SizedUnit {
    type Output = Self;

    /// Multiplies two sized units together
    ///
    /// For example, `(10 g) * (1000 g) = (10000 g^2)`
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            magnitude: self.magnitude * rhs.magnitude,
            unit: self.unit * rhs.unit,
            is_db: self.is_db || rhs.is_db,
        }
    }
}

impl ops::Div for SizedUnit {
    type Output = Self;

    /// Divides two sized units
    ///
    /// For example, `(10000 g^2) / (10 g) = (1000 g)`
    fn div(self, rhs: Self) -> Self::Output {
        Self {
            magnitude: self.magnitude / rhs.magnitude,
            unit: self.unit / rhs.unit,
            is_db: self.is_db || rhs.is_db,
        }
    }
}
