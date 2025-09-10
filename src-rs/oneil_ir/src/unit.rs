//! Unit system for dimensional analysis in Oneil.

use crate::span::IrSpan;

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
    name_span: IrSpan,
    exponent: f64,
    exponent_span: Option<IrSpan>,
}

impl Unit {
    /// Creates a new unit with the specified name and exponent.
    #[must_use]
    pub const fn new(
        name: String,
        name_span: IrSpan,
        exponent: f64,
        exponent_span: Option<IrSpan>,
    ) -> Self {
        Self {
            name,
            name_span,
            exponent,
            exponent_span,
        }
    }

    /// Returns the name of this unit.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the source location span for the unit name.
    #[must_use]
    pub const fn name_span(&self) -> IrSpan {
        self.name_span
    }

    /// Returns the exponent of this unit.
    #[must_use]
    pub const fn exponent(&self) -> f64 {
        self.exponent
    }

    /// Returns the source location span for the unit exponent.
    #[must_use]
    pub const fn exponent_span(&self) -> Option<IrSpan> {
        self.exponent_span
    }
}
