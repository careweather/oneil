use std::{fmt, ops};

use indexmap::IndexMap;

use crate::value::util::is_close;

/// A unit in Oneil.
///
/// A unit has two parts: a dimension map and display information.
///
/// Units should never be compared for equality. If you are looking
/// for equality, you probably actually want to check if the dimension maps
/// are equal. If you would like to check if two units are exactly the same,
/// check the individual components for equality.
#[derive(Debug, Clone)]
pub struct Unit {
    /// The dimensions of the unit
    pub dimension_map: DimensionMap,
    /// The magnitude of the unit (e.g. 1000 for km)
    pub magnitude: f64,
    /// Whether the unit is a decibel unit
    pub is_db: bool,
    /// The display information for the unit
    pub display_unit: DisplayUnit,
}

impl Unit {
    /// Creates a unitless unit.
    #[must_use]
    pub fn unitless() -> Self {
        Self {
            dimension_map: DimensionMap::unitless(),
            magnitude: 1.0,
            is_db: false,
            display_unit: DisplayUnit::Unitless,
        }
    }

    /// Determines if the unit is unitless.
    #[must_use]
    pub fn is_unitless(&self) -> bool {
        self.dimension_map.is_unitless()
    }

    /// Sets the `is_db` flag for the unit.
    #[must_use]
    pub fn with_is_db_as(self, is_db: bool) -> Self {
        Self { is_db, ..self }
    }

    /// Sets the unit display expression for the unit.
    #[must_use]
    pub fn with_unit_display_expr(self, display_expr: DisplayUnit) -> Self {
        Self {
            display_unit: display_expr,
            ..self
        }
    }

    /// Multiplies the unit by the given magnitude.
    #[must_use]
    pub fn mul_magnitude(self, magnitude: f64) -> Self {
        Self {
            magnitude: self.magnitude * magnitude,
            ..self
        }
    }

    /// Raises the unit to the power of the given exponent.
    #[must_use]
    pub fn pow(self, exponent: f64) -> Self {
        Self {
            dimension_map: self.dimension_map.pow(exponent),
            magnitude: self.magnitude.powf(exponent),
            is_db: self.is_db,
            display_unit: self.display_unit.pow(exponent),
        }
    }

    /// Determines if the unit has the same dimensions as the
    /// given dimension map.
    #[must_use]
    pub fn dimensions_match(&self, dimension_map: &DimensionMap) -> bool {
        self.dimension_map == *dimension_map
    }

    /// Determines if the unit has the same dimensions as the given unit.
    ///
    /// For example, according to dimensionally equality, `km == m` because
    /// they have the same dimensions, while `km != km/h` because they have
    /// different dimensions.
    #[must_use]
    pub fn dimensionally_eq(&self, other: &Self) -> bool {
        self.dimension_map == other.dimension_map
    }

    /// Determines if the unit is numerically equal to the given unit.
    ///
    /// For example, according to numerically equality, `km == km` but
    /// `km != m` because the magnitudes are different.
    ///
    /// This includes dimensions, magnitude, and `is_db`.
    ///
    /// NOTE: I wasn't sure exactly how to name this function,
    ///       this is the best name I could come up with.
    #[must_use]
    pub fn numerically_eq(&self, other: &Self) -> bool {
        self.magnitude == other.magnitude
            && self.is_db == other.is_db
            && self.dimensionally_eq(other)
    }
}

impl ops::Mul for Unit {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            dimension_map: self.dimension_map * rhs.dimension_map,
            magnitude: self.magnitude * rhs.magnitude,
            is_db: self.is_db || rhs.is_db,
            display_unit: self.display_unit * rhs.display_unit,
        }
    }
}

impl ops::Div for Unit {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            dimension_map: self.dimension_map / rhs.dimension_map,
            magnitude: self.magnitude / rhs.magnitude,
            is_db: self.is_db || rhs.is_db,
            display_unit: self.display_unit / rhs.display_unit,
        }
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_unit)
    }
}
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

/// A map of dimensions and their exponents.
///
/// For example, "m/s" is represented as `DimensionMap(IndexMap::from([(Dimension::Distance, 1.0), (Dimension::Time, -1.0)]))`.
#[derive(Debug, Clone)]
pub struct DimensionMap(IndexMap<Dimension, f64>);

impl DimensionMap {
    /// Creates a new unit from a map of dimensions and their exponents.
    #[must_use]
    pub const fn new(units: IndexMap<Dimension, f64>) -> Self {
        Self(units)
    }

    /// Creates a unitless unit, which is a unit with no dimensions.
    #[must_use]
    pub fn unitless() -> Self {
        Self(IndexMap::new())
    }

    /// Checks if the unit is unitless (has no dimensions).
    #[must_use]
    pub fn is_unitless(&self) -> bool {
        self.0.is_empty()
    }

    /// Raises the unit to the power of the given exponent.
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

impl PartialEq for DimensionMap {
    /// Checks if two units are equal
    ///
    /// Note that this is a fuzzy equality check, and
    /// that the units are considered equal if their
    /// dimensions and exponents are close, as determined
    /// by the `is_close` function.
    fn eq(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }

        self.0.iter().all(|(dimension, value)| {
            let other_value = other.0.get(dimension);
            other_value.is_some_and(|other_value| is_close(*other_value, *value))
        })
    }
}

impl ops::Mul for DimensionMap {
    type Output = Self;

    /// Multiplies two units together
    ///
    /// For example, `(m/s) * (g) = (g*m/s)`
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

impl ops::Div for DimensionMap {
    type Output = Self;

    /// Divides two units
    ///
    /// For example, `(g*m/s) / (g) = (m/s)`
    fn div(self, rhs: Self) -> Self::Output {
        let mut result = self.0;

        for (key, value) in rhs.0 {
            #[expect(
                clippy::suspicious_arithmetic_impl,
                reason = "division is defined as subtraction of the exponent"
            )]
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

/// A display unit in Oneil.
///
/// A display unit is a unit that is displayed to the user.
/// It is used to represent the unit in a human-readable format.
///
/// It uses an AST-like structure.
#[derive(Debug, Clone, PartialEq)]
pub enum DisplayUnit {
    /// Unitless `1`
    Unitless,
    /// A single unit
    Unit {
        /// The name of the unit
        name: String,
        /// The exponent of the unit
        exponent: f64,
    },
    /// A multiplied unit
    Multiply(Box<DisplayUnit>, Box<DisplayUnit>),
    /// A divided unit
    Divide(Box<DisplayUnit>, Box<DisplayUnit>),
    /// A power unit
    Power {
        /// The base of the power unit
        base: Box<DisplayUnit>,
        /// The exponent of the power unit
        exponent: f64,
    },
}

impl DisplayUnit {
    /// Raises the display unit to the power of the given exponent.
    #[must_use]
    pub fn pow(self, pow_exponent: f64) -> Self {
        match self {
            Self::Unitless => Self::Unitless,
            Self::Unit { name, exponent } => Self::Unit {
                name,
                exponent: exponent * pow_exponent,
            },
            Self::Multiply(_, _) | Self::Divide(_, _) => Self::Power {
                base: Box::new(self),
                exponent: pow_exponent,
            },
            Self::Power { base, exponent } => Self::Power {
                base,
                exponent: exponent * pow_exponent,
            },
        }
    }
}

impl ops::Mul for DisplayUnit {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::Multiply(Box::new(self), Box::new(rhs))
    }
}

impl ops::Div for DisplayUnit {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self::Divide(Box::new(self), Box::new(rhs))
    }
}

impl fmt::Display for DisplayUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unitless => write!(f, "1")?,
            Self::Unit { name, exponent } => {
                write!(f, "{name}")?;
                if !is_close(*exponent, 1.0) {
                    write!(f, "^{exponent}")?;
                }
            }
            Self::Multiply(left, right) => write!(f, "{left}*{right}")?,
            Self::Divide(left, right) => match **right {
                Self::Multiply(_, _) | Self::Divide(_, _) => write!(f, "{left}/({right})")?,
                Self::Unitless | Self::Unit { .. } | Self::Power { .. } => {
                    write!(f, "{left}/{right}")?;
                }
            },
            Self::Power { base, exponent } => {
                match **base {
                    Self::Unitless | Self::Unit { .. } => write!(f, "({base})^{exponent}")?,
                    Self::Multiply(_, _) | Self::Divide(_, _) | Self::Power { .. } => {
                        write!(f, "({base})^{exponent}")?;
                    }
                }
                write!(f, "({base})^{exponent}")?;
            }
        }

        Ok(())
    }
}
