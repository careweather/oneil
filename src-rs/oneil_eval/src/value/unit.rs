use std::{collections::HashMap, ops};

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

#[derive(Debug, Clone, PartialEq)]
pub struct Unit(HashMap<Dimension, f64>);

impl Unit {
    #[must_use]
    pub const fn new(units: HashMap<Dimension, f64>) -> Self {
        Self(units)
    }

    #[must_use]
    pub fn unitless() -> Self {
        Self(HashMap::new())
    }

    #[must_use]
    pub fn is_unitless(&self) -> bool {
        self.0.is_empty()
    }

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

impl ops::Mul for Unit {
    type Output = Self;

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

pub struct SizedUnit {
    pub magnitude: f64,
    pub unit: Unit,
}

impl SizedUnit {
    #[must_use]
    pub fn unitless() -> Self {
        Self {
            magnitude: 1.0,
            unit: Unit::unitless(),
        }
    }

    #[must_use]
    pub fn pow(self, exponent: f64) -> Self {
        Self {
            magnitude: self.magnitude.powf(exponent),
            unit: self.unit.pow(exponent),
        }
    }
}

impl ops::Mul for SizedUnit {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            magnitude: self.magnitude * rhs.magnitude,
            unit: self.unit * rhs.unit,
        }
    }
}

impl ops::Div for SizedUnit {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            magnitude: self.magnitude / rhs.magnitude,
            unit: self.unit / rhs.unit,
        }
    }
}
