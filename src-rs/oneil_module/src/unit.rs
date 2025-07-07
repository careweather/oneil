#[derive(Debug, Clone, PartialEq)]
pub struct CompositeUnit {
    units: Vec<Unit>,
}

impl CompositeUnit {
    pub fn new(units: Vec<Unit>) -> Self {
        Self { units }
    }

    /// Returns a reference to the units in this composite unit
    pub fn units(&self) -> &[Unit] {
        &self.units
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Unit {
    name: String,
    exponent: f64,
}

impl Unit {
    pub fn new(name: String, exponent: f64) -> Self {
        Self { name, exponent }
    }

    /// Returns the name of this unit
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the exponent of this unit
    pub fn exponent(&self) -> f64 {
        self.exponent
    }
}
