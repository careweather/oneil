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
pub struct ComplexDimension(HashMap<Dimension, f64>);

impl ComplexDimension {
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

impl ops::Mul for ComplexDimension {
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

impl ops::Div for ComplexDimension {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let mut result = self.0;

        for (key, value) in rhs.0 {
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

/*
#[derive(Debug, Clone, PartialEq)]
pub struct Unit {
    dimensions: ComplexDimension,
    magnitude: f64,
}

impl Unit {
    /// Creates a new unit with the given dimensions and magnitude.
    ///
    /// # Panics
    ///
    /// Panics if the magnitude is zero or negative.
    #[must_use]
    pub const fn new(dimensions: ComplexDimension, magnitude: f64) -> Self {
        assert!(magnitude > 0.0, "magnitude must be positive");
        Self {
            dimensions,
            magnitude,
        }
    }

    #[must_use]
    pub const fn dimensions(&self) -> &ComplexDimension {
        &self.dimensions
    }

    #[must_use]
    pub const fn magnitude(&self) -> f64 {
        self.magnitude
    }

    #[must_use]
    pub fn pow(&self, exponent: f64) -> Self {
        todo!()
    }
}

impl ops::Mul for Unit {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl ops::Div for Unit {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

*/
