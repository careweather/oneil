use std::collections::HashMap;

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
}

#[derive(Debug, Clone, PartialEq)]
pub struct Unit {
    dimensions: ComplexDimension,
    magnitude: f64,
}

impl Unit {
    #[must_use]
    pub const fn new(dimensions: ComplexDimension, magnitude: f64) -> Self {
        Self {
            dimensions,
            magnitude,
        }
    }

    #[must_use]
    pub const fn dimensions(&self) -> &ComplexDimension {
        &self.dimensions
    }
}
