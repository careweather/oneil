//! Unit system for dimensional analysis in Oneil.

/// A composite unit composed of multiple base units.
#[derive(Debug, Clone, PartialEq)]
pub struct CompositeUnit {
    units: Vec<Unit>,
}

impl CompositeUnit {
    /// Creates a new composite unit from a vector of individual units.
    #[must_use]
    pub const fn new(units: Vec<Unit>) -> Self {
        Self { units }
    }

    /// Returns a reference to the units in this composite unit.
    #[must_use]
    pub fn units(&self) -> &[Unit] {
        &self.units
    }
}

/// A single unit with a name and exponent.
#[derive(Debug, Clone, PartialEq)]
pub struct Unit {
    name: String,
    exponent: f64,
}

impl Unit {
    /// Creates a new unit with the specified name and exponent.
    #[must_use]
    pub const fn new(name: String, exponent: f64) -> Self {
        Self { name, exponent }
    }

    /// Returns the name of this unit.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the exponent of this unit.
    #[must_use]
    pub const fn exponent(&self) -> f64 {
        self.exponent
    }
}
