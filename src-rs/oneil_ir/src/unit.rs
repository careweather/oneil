//! Unit system for dimensional analysis in Oneil.
//!
//! This module provides the data structures for representing physical units
//! in Oneil. Units are used for dimensional analysis, ensuring that
//! calculations are physically meaningful and preventing unit-related errors.
//!
//! The unit system supports both simple units and composite units that
//! combine multiple base units with exponents.

use crate::span::IrSpan;

/// A composite unit composed of multiple base units.
///
/// `CompositeUnit` represents a complex unit that is built from multiple
/// base units, each with their own exponent. This allows representation
/// of units like "m²·kg/s²" (newtons) or "kg·m²/s²" (joules).
///
/// Composite units are immutable and can be easily cloned for manipulation.
#[derive(Debug, Clone, PartialEq)]
pub struct CompositeUnit {
    units: Vec<Unit>,
}

impl CompositeUnit {
    /// Creates a new composite unit from a vector of individual units.
    ///
    /// # Arguments
    ///
    /// * `units` - Vector of units that make up this composite unit
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{unit::{CompositeUnit, Unit}, span::Span};
    ///
    /// let units = vec![
    ///     Unit::new("m".to_string(), Span::new(0, 1), 2.0, Span::new(0, 1)),  // meters squared
    ///     Unit::new("kg".to_string(), Span::new(0, 2), 1.0, Span::new(0, 1)), // kilograms
    ///     Unit::new("s".to_string(), Span::new(0, 1), -2.0, Span::new(0, 1)), // per second squared
    /// ];
    ///
    /// let composite = CompositeUnit::new(units);
    /// // Represents: m²·kg/s² (newtons)
    /// ```
    #[must_use]
    pub const fn new(units: Vec<Unit>) -> Self {
        Self { units }
    }

    /// Returns a reference to the units in this composite unit.
    ///
    /// This method provides access to the individual units that make up
    /// the composite unit, allowing inspection of each unit's name and exponent.
    ///
    /// # Returns
    ///
    /// A slice containing references to all units in this composite unit.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{unit::{CompositeUnit, Unit}, span::Span};
    ///
    /// let units = vec![
    ///     Unit::new("m".to_string(), Span::new(0, 1), 1.0, Span::new(0, 1)),
    ///     Unit::new("s".to_string(), Span::new(0, 1), -1.0, Span::new(0, 1)),
    /// ];
    ///
    /// let composite = CompositeUnit::new(units);
    /// let unit_refs = composite.units();
    ///
    /// assert_eq!(unit_refs.len(), 2);
    /// assert_eq!(unit_refs[0].name(), "m");
    /// assert_eq!(unit_refs[0].exponent(), 1.0);
    /// assert_eq!(unit_refs[1].name(), "s");
    /// assert_eq!(unit_refs[1].exponent(), -1.0);
    /// ```
    #[must_use]
    pub fn units(&self) -> &[Unit] {
        &self.units
    }
}

/// A single unit with a name and exponent.
///
/// `Unit` represents a base unit in the Oneil unit system. Each unit has:
///
/// - **Name**: The symbol or name of the unit (e.g., "m", "kg", "s")
/// - **Exponent**: The power to which this unit is raised
///
/// Units can have positive exponents (e.g., m²), negative exponents (e.g., s⁻¹),
/// or fractional exponents (e.g., m^0.5).
#[derive(Debug, Clone, PartialEq)]
pub struct Unit {
    name: String,
    name_span: IrSpan,
    exponent: f64,
    exponent_span: IrSpan,
}

impl Unit {
    /// Creates a new unit with the specified name and exponent.
    ///
    /// # Arguments
    ///
    /// * `name` - The name or symbol of the unit
    /// * `exponent` - The exponent for this unit
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{unit::Unit, span::Span};
    ///
    /// let meter = Unit::new("m".to_string(), Span::new(0, 1), 1.0, Span::new(0, 1));
    /// let square_meter = Unit::new("m".to_string(), Span::new(0, 1), 2.0, Span::new(0, 1));
    /// let per_second = Unit::new("s".to_string(), Span::new(0, 1), -1.0, Span::new(0, 1));
    /// let sqrt_meter = Unit::new("m".to_string(), Span::new(0, 1), 0.5, Span::new(0, 1));
    /// ```
    #[must_use]
    pub const fn new(
        name: String,
        name_span: IrSpan,
        exponent: f64,
        exponent_span: IrSpan,
    ) -> Self {
        Self {
            name,
            name_span,
            exponent,
            exponent_span,
        }
    }

    /// Returns the name of this unit.
    ///
    /// The name is typically a standard unit symbol like "m" for meters,
    /// "kg" for kilograms, or "s" for seconds.
    ///
    /// # Returns
    ///
    /// A string slice containing the unit's name.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{unit::Unit, span::Span};
    ///
    /// let unit = Unit::new("kg".to_string(), Span::new(0, 2), 1.0, Span::new(0, 1));
    /// assert_eq!(unit.name(), "kg");
    /// ```
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the source location span for the unit name.
    ///
    /// This method provides access to the source location information
    /// for the unit name, which is useful for error reporting and
    /// debugging.
    ///
    /// # Returns
    ///
    /// A reference to the `Span` indicating where the unit name appears in the source.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{unit::Unit, span::Span};
    ///
    /// let name_span = Span::new(10, 2);
    /// let unit = Unit::new("kg".to_string(), name_span, 1.0, Span::new(0, 1));
    /// assert_eq!(unit.name_span().start(), 10);
    /// assert_eq!(unit.name_span().length(), 2);
    /// ```
    #[must_use]
    pub const fn name_span(&self) -> IrSpan {
        self.name_span
    }

    /// Returns the exponent of this unit.
    ///
    /// The exponent determines the power to which this unit is raised.
    /// Common values include:
    /// - `1.0` for linear units (e.g., meters)
    /// - `2.0` for squared units (e.g., square meters)
    /// - `-1.0` for inverse units (e.g., per second)
    /// - `0.5` for square root units (e.g., √meters)
    ///
    /// # Returns
    ///
    /// The exponent as a floating-point number.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{unit::Unit, span::Span};
    ///
    /// let linear = Unit::new("m".to_string(), Span::new(0, 1), 1.0, Span::new(0, 1));
    /// let squared = Unit::new("m".to_string(), Span::new(0, 1), 2.0, Span::new(0, 1));
    /// let inverse = Unit::new("s".to_string(), Span::new(0, 1), -1.0, Span::new(0, 1));
    ///
    /// assert_eq!(linear.exponent(), 1.0);
    /// assert_eq!(squared.exponent(), 2.0);
    /// assert_eq!(inverse.exponent(), -1.0);
    /// ```
    #[must_use]
    pub const fn exponent(&self) -> f64 {
        self.exponent
    }

    /// Returns the source location span for the unit exponent.
    ///
    /// This method provides access to the source location information
    /// for the unit exponent, which is useful for error reporting and
    /// debugging.
    ///
    /// # Returns
    ///
    /// A reference to the `Span` indicating where the unit exponent appears in the source.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{unit::Unit, span::Span};
    ///
    /// let exponent_span = Span::new(15, 3);
    /// let unit = Unit::new("m".to_string(), Span::new(0, 1), 2.0, exponent_span);
    /// assert_eq!(unit.exponent_span().start(), 15);
    /// assert_eq!(unit.exponent_span().length(), 3);
    /// ```
    #[must_use]
    pub const fn exponent_span(&self) -> IrSpan {
        self.exponent_span
    }
}
