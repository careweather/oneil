//! Unit system for dimensional analysis in Oneil.
//!
//! This module provides the data structures for representing physical units
//! in Oneil. Units are used for dimensional analysis, ensuring that
//! calculations are physically meaningful and preventing unit-related errors.
//!
//! The unit system supports both simple units and composite units that
//! combine multiple base units with exponents.

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
    /// use oneil_ir::unit::{CompositeUnit, Unit};
    ///
    /// let units = vec![
    ///     Unit::new("m".to_string(), 2.0),  // meters squared
    ///     Unit::new("kg".to_string(), 1.0), // kilograms
    ///     Unit::new("s".to_string(), -2.0), // per second squared
    /// ];
    ///
    /// let composite = CompositeUnit::new(units);
    /// // Represents: m²·kg/s² (newtons)
    /// ```
    pub fn new(units: Vec<Unit>) -> Self {
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
    /// use oneil_ir::unit::{CompositeUnit, Unit};
    ///
    /// let units = vec![
    ///     Unit::new("m".to_string(), 1.0),
    ///     Unit::new("s".to_string(), -1.0),
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
    exponent: f64,
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
    /// use oneil_ir::unit::Unit;
    ///
    /// let meter = Unit::new("m".to_string(), 1.0);
    /// let square_meter = Unit::new("m".to_string(), 2.0);
    /// let per_second = Unit::new("s".to_string(), -1.0);
    /// let sqrt_meter = Unit::new("m".to_string(), 0.5);
    /// ```
    pub fn new(name: String, exponent: f64) -> Self {
        Self { name, exponent }
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
    /// use oneil_ir::unit::Unit;
    ///
    /// let unit = Unit::new("kg".to_string(), 1.0);
    /// assert_eq!(unit.name(), "kg");
    /// ```
    pub fn name(&self) -> &str {
        &self.name
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
    /// use oneil_ir::unit::Unit;
    ///
    /// let linear = Unit::new("m".to_string(), 1.0);
    /// let squared = Unit::new("m".to_string(), 2.0);
    /// let inverse = Unit::new("s".to_string(), -1.0);
    ///
    /// assert_eq!(linear.exponent(), 1.0);
    /// assert_eq!(squared.exponent(), 2.0);
    /// assert_eq!(inverse.exponent(), -1.0);
    /// ```
    pub fn exponent(&self) -> f64 {
        self.exponent
    }
}
