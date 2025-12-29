//! Unit system for dimensional analysis in Oneil.

/// A composite unit composed of multiple base units.
#[derive(Debug, Clone, PartialEq)]
pub struct CompositeUnit {
    units: Vec<Unit>,
    display_unit: DisplayCompositeUnit,
}

impl CompositeUnit {
    /// Creates a new composite unit from a vector of individual units.
    #[must_use]
    pub const fn new(units: Vec<Unit>, display_unit: DisplayCompositeUnit) -> Self {
        Self {
            units,
            display_unit,
        }
    }

    /// Returns a reference to the units in this composite unit.
    #[must_use]
    pub fn units(&self) -> &[Unit] {
        &self.units
    }

    /// Returns a reference to the display unit of this composite unit.
    #[must_use]
    pub fn display_unit(&self) -> &DisplayCompositeUnit {
        &self.display_unit
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

/// A unit used for displaying the unit to
/// the user.
///
/// This retains multiplication and division and
/// the original exponent, rather than converting
/// it to a list of units that are multiplied
/// together.
#[derive(Debug, Clone, PartialEq)]
pub enum DisplayCompositeUnit {
    /// Multiplied units
    Multiply(Box<DisplayCompositeUnit>, Box<DisplayCompositeUnit>),
    /// Divided units
    Divide(Box<DisplayCompositeUnit>, Box<DisplayCompositeUnit>),
    /// A single unit
    BaseUnit(Unit),
    /// Unitless `1`
    Unitless,
}
