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

    pub fn unitless() -> Self {
        Self(HashMap::new())
    }

    pub fn is_unitless(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Unit {
    dimensions: ComplexDimension,
    magnitude: f64,
}

impl Unit {
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
