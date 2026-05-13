use std::collections::BTreeMap;
use std::{fmt, ops};

use serde::de::{self, Deserializer};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

use crate::util::is_close;

/// A unit in Oneil.
///
/// A unit has two parts: a dimension map and display information.
///
/// Units should never be compared for equality. If you are looking
/// for equality, you probably actually want to check if the dimension maps
/// are equal. If you would like to check if two units are exactly the same,
/// check the individual components for equality.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Unit {
    /// The dimensions of the unit
    #[serde(rename = "dimensions")]
    pub dimension_map: DimensionMap,
    /// The magnitude of the unit (e.g. 1000 for km)
    pub magnitude: f64,
    /// Whether the unit is a decibel unit
    pub is_db: bool,
    /// The display information for the unit
    pub display_unit: DisplayUnit,
}

impl Unit {
    /// Creates a `1` unit.
    ///
    /// This is a unit with no dimensions and a
    /// magnitude of 1. It is also not a decibel unit.
    #[must_use]
    pub const fn one() -> Self {
        Self {
            dimension_map: DimensionMap::dimensionless(),
            magnitude: 1.0,
            is_db: false,
            display_unit: DisplayUnit::One,
        }
    }

    /// Determines if the unit is dimensionless.
    #[must_use]
    pub fn is_dimensionless(&self) -> bool {
        self.dimension_map.is_dimensionless()
    }

    /// Determines if the unit is effectively unitless.
    ///
    /// A unit is effectively unitless when its magnitude is 1, it is not a dB unit,
    /// and it is dimensionless.
    #[must_use]
    pub fn is_effectively_unitless(&self) -> bool {
        is_close(self.magnitude, 1.0) && !self.is_db && self.dimension_map.is_dimensionless()
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

impl Dimension {
    /// SI-style key for this dimension (used in interchange maps).
    #[must_use]
    pub const fn as_map_key(self) -> &'static str {
        match self {
            Self::Mass => "kg",
            Self::Distance => "m",
            Self::Time => "s",
            Self::Temperature => "K",
            Self::Current => "A",
            Self::Information => "bit",
            Self::Currency => "$",
            Self::Substance => "mol",
            Self::LuminousIntensity => "cd",
        }
    }

    /// Parses a dimension from an interchange map key (see [`Self::as_map_key`]).
    #[must_use]
    pub fn from_map_key(s: &str) -> Option<Self> {
        match s {
            "kg" => Some(Self::Mass),
            "m" => Some(Self::Distance),
            "s" => Some(Self::Time),
            "K" => Some(Self::Temperature),
            "A" => Some(Self::Current),
            "bit" => Some(Self::Information),
            "$" => Some(Self::Currency),
            "mol" => Some(Self::Substance),
            "cd" => Some(Self::LuminousIntensity),
            _ => None,
        }
    }
}

impl Serialize for Dimension {
    /// Serializes as the interchange string key (see [`Dimension::as_map_key`]).
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_map_key())
    }
}

impl<'de> Deserialize<'de> for Dimension {
    /// Deserializes from an interchange string key (see [`Dimension::from_map_key`]).
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_map_key(&s).ok_or_else(|| {
            de::Error::unknown_variant(&s, &["kg", "m", "s", "K", "A", "bit", "$", "mol", "cd"])
        })
    }
}

/// A map of dimensions and their exponents.
///
/// Stored as a [`BTreeMap`] so iteration and JSON key order are always deterministic
/// (ordered by [`Dimension`], not insertion order).
///
/// For example, "m/s" is represented as
/// `DimensionMap(BTreeMap::from([(Dimension::Distance, 1.0), (Dimension::Time, -1.0)]))`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DimensionMap(BTreeMap<Dimension, f64>);

impl DimensionMap {
    /// Creates a new dimension map from a map of dimensions and their exponents.
    #[must_use]
    pub const fn new(units: BTreeMap<Dimension, f64>) -> Self {
        Self(units)
    }

    /// Creates a dimensionless map
    #[must_use]
    pub const fn dimensionless() -> Self {
        Self(BTreeMap::new())
    }

    /// Checks if the dimension map is dimensionless
    #[must_use]
    pub fn is_dimensionless(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns a reference to the underlying map of dimension to exponent.
    #[must_use]
    pub const fn as_map(&self) -> &BTreeMap<Dimension, f64> {
        &self.0
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

impl FromIterator<(Dimension, f64)> for DimensionMap {
    fn from_iter<I: IntoIterator<Item = (Dimension, f64)>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
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
    /// `1` unit
    One,
    /// A single unit
    Unit {
        /// The name of the unit
        name: String,
        /// The exponent of the unit
        exponent: f64,
    },
    /// A multiplied unit
    Multiply(Box<Self>, Box<Self>),
    /// A divided unit
    Divide(Box<Self>, Box<Self>),
    /// A power unit
    Power {
        /// The base of the power unit
        base: Box<Self>,
        /// The exponent of the power unit
        exponent: f64,
    },
    /// A unit with its contents rendered as a string
    ///
    /// Mainly used for serialization/deserialization.
    RenderedUnit(String),
}

impl DisplayUnit {
    /// Raises the display unit to the power of the given exponent.
    #[must_use]
    pub fn pow(self, pow_exponent: f64) -> Self {
        match self {
            Self::One => Self::One,
            Self::Unit { name, exponent } => Self::Unit {
                name,
                exponent: exponent * pow_exponent,
            },
            Self::Multiply(_, _) | Self::Divide(_, _) | Self::RenderedUnit(_) => Self::Power {
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
            Self::One => write!(f, "1")?,
            Self::Unit { name, exponent } => {
                write!(f, "{name}")?;
                if !is_close(*exponent, 1.0) {
                    write!(f, "^{exponent}")?;
                }
            }
            Self::Multiply(left, right) if **left == Self::One => write!(f, "{right}")?,
            Self::Multiply(left, right) if **right == Self::One => write!(f, "{left}")?,
            Self::Multiply(left, right) => write!(f, "{left}*{right}")?,
            Self::Divide(left, right) => match **right {
                Self::Multiply(_, _) | Self::Divide(_, _) | Self::RenderedUnit(_) => {
                    write!(f, "{left}/({right})")?;
                }
                Self::One => write!(f, "{left}")?,
                Self::Unit { .. } | Self::Power { .. } => {
                    write!(f, "{left}/{right}")?;
                }
            },
            Self::Power { base, exponent } => match **base {
                Self::One => write!(f, "{base}^{exponent}")?,
                Self::Unit {
                    exponent: base_exponent,
                    ..
                } if is_close(base_exponent, 1.0) => write!(f, "{base}^{exponent}")?,
                Self::Unit { .. }
                | Self::Multiply(_, _)
                | Self::Divide(_, _)
                | Self::Power { .. }
                | Self::RenderedUnit(_) => {
                    write!(f, "({base})^{exponent}")?;
                }
            },
            Self::RenderedUnit(contents) => write!(f, "{contents}")?,
        }

        Ok(())
    }
}

impl Serialize for DisplayUnit {
    /// Serializes as the same string produced by [`fmt::Display`].
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for DisplayUnit {
    /// Deserializes a string into a single named unit with exponent `1.0`.
    ///
    /// This is to avoid re-parsing the display unit string.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let name = String::deserialize(deserializer)?;
        Ok(Self::RenderedUnit(name))
    }
}

#[cfg(test)]
mod serde_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn dimension_map_json_round_trip() {
        let m = DimensionMap::new(BTreeMap::from([
            (Dimension::Mass, 1.0),
            (Dimension::Distance, 2.0),
            (Dimension::Time, -3.0),
        ]));
        let val = serde_json::to_value(&m).expect("serialize");
        assert_eq!(val["kg"], json!(1.0));
        assert_eq!(val["m"], json!(2.0));
        assert_eq!(val["s"], json!(-3.0));
        let back: DimensionMap = serde_json::from_value(val).expect("deserialize");
        assert_eq!(back, m);
    }

    #[test]
    fn dimension_map_json_key_ordering_is_sorted() {
        let m = DimensionMap::new(BTreeMap::from([
            (Dimension::Time, 1.0),
            (Dimension::Mass, 2.0),
        ]));
        let keys: Vec<String> = serde_json::to_value(&m)
            .expect("serialize")
            .as_object()
            .expect("object")
            .keys()
            .cloned()
            .collect();
        assert_eq!(keys, vec!["kg".to_string(), "s".to_string()]);
    }

    #[test]
    fn display_unit_json_round_trip() {
        let d = DisplayUnit::Unit {
            name: "N".into(),
            exponent: 1.0,
        };
        let json = serde_json::to_string(&d).expect("serialize");
        assert_eq!(json, "\"N\"");
        let back: DisplayUnit = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, DisplayUnit::RenderedUnit(d.to_string()));
    }

    #[test]
    fn unit_json_round_trip() {
        let u = Unit {
            dimension_map: DimensionMap::new(BTreeMap::from([(Dimension::Distance, 1.0)])),
            magnitude: 1.0,
            is_db: false,
            display_unit: DisplayUnit::Unit {
                name: "m".into(),
                exponent: 1.0,
            },
        };
        let json = serde_json::to_string(&u).expect("serialize");
        let back: Unit = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.dimension_map, u.dimension_map);
        assert!((back.magnitude - u.magnitude).abs() < f64::EPSILON);
        assert_eq!(back.is_db, u.is_db);
        assert_eq!(
            back.display_unit,
            DisplayUnit::RenderedUnit(u.display_unit.to_string())
        );
    }

    #[test]
    fn unit_with_exponent_json_round_trip() {
        let u = Unit {
            dimension_map: DimensionMap::new(BTreeMap::from([(Dimension::Distance, 2.0)])),
            magnitude: 1000.0,
            is_db: false,
            display_unit: DisplayUnit::Unit {
                name: "km".into(),
                exponent: 2.0,
            },
        };
        let json = serde_json::to_string(&u).expect("serialize");
        let back: Unit = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.dimension_map, u.dimension_map);
        assert!((back.magnitude - u.magnitude).abs() < f64::EPSILON);
        assert_eq!(back.is_db, u.is_db);
        assert_eq!(
            back.display_unit,
            DisplayUnit::RenderedUnit(u.display_unit.to_string())
        );
    }
}
