//! Unit system for dimensional analysis in Oneil.

use oneil_shared::span::Span;

/// A composite unit composed of multiple base units.
#[derive(Debug, Clone, PartialEq)]
pub struct CompositeUnit {
    units: Vec<Unit>,
    display_unit: DisplayCompositeUnit,
    span: Span,
}

impl CompositeUnit {
    /// Creates a new composite unit from a vector of individual units.
    #[must_use]
    pub const fn new(units: Vec<Unit>, display_unit: DisplayCompositeUnit, span: Span) -> Self {
        Self {
            units,
            display_unit,
            span,
        }
    }

    /// Returns a reference to the units in this composite unit.
    #[must_use]
    pub const fn units(&self) -> &[Unit] {
        self.units.as_slice()
    }

    /// Returns a reference to the display unit of this composite unit.
    #[must_use]
    pub const fn display_unit(&self) -> &DisplayCompositeUnit {
        &self.display_unit
    }

    /// Returns the span of this composite unit.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }
}

/// A single unit with a name and exponent.
#[derive(Debug, Clone, PartialEq)]
pub struct Unit {
    span: Span,
    name: String,
    name_span: Span,
    exponent: f64,
    exponent_span: Option<Span>,
}

impl Unit {
    /// Creates a new unit with the specified name and exponent.
    #[must_use]
    pub const fn new(
        span: Span,
        name: String,
        name_span: Span,
        exponent: f64,
        exponent_span: Option<Span>,
    ) -> Self {
        Self {
            span,
            name,
            name_span,
            exponent,
            exponent_span,
        }
    }

    /// Returns the span of this unit.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the name of this unit.
    #[must_use]
    pub const fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the span of the name of this unit.
    #[must_use]
    pub const fn name_span(&self) -> Span {
        self.name_span
    }

    /// Returns the exponent of this unit.
    #[must_use]
    pub const fn exponent(&self) -> f64 {
        self.exponent
    }

    /// Returns the span of the exponent of this unit.
    #[must_use]
    pub const fn exponent_span(&self) -> Option<Span> {
        self.exponent_span
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
    BaseUnit(DisplayUnit),
    /// Unitless `1`
    Unitless,
}

/// A unit used for displaying the unit to
/// the user.
///
/// This retains multiplication and division and
/// the original exponent, rather than converting
/// it to a list of units that are multiplied
/// together.
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayUnit {
    /// The name of the unit
    pub name: String,
    /// The exponent of the unit
    pub exponent: f64,
}

impl DisplayUnit {
    /// Creates a new display unit with the specified name and exponent.
    #[must_use]
    pub const fn new(name: String, exponent: f64) -> Self {
        Self { name, exponent }
    }
}
