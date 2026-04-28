//! Serialized cache values and conversions to [`oneil_output::Value`].

use std::collections::BTreeMap;
use std::fmt;

use indexmap::IndexMap;
use oneil_output::{
    Dimension, DimensionMap, DisplayUnit, MeasuredNumber, Number, Unit as OutputUnit, Value,
};
use serde::de::{self, Deserializer};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

/// Wrapper around [`Value`] for JSON serialization and deserialization.
#[derive(Debug, Clone, PartialEq)]
pub struct CacheValue(pub Value);

impl From<Value> for CacheValue {
    fn from(value: Value) -> Self {
        Self(value)
    }
}

impl From<CacheValue> for Value {
    fn from(value: CacheValue) -> Self {
        value.0
    }
}

impl PartialEq<Value> for CacheValue {
    fn eq(&self, other: &Value) -> bool {
        self.0 == *other
    }
}

impl PartialEq<CacheValue> for Value {
    fn eq(&self, other: &CacheValue) -> bool {
        *self == other.0
    }
}

impl PartialEq<&Value> for CacheValue {
    fn eq(&self, other: &&Value) -> bool {
        self.0 == **other
    }
}

impl PartialEq<&CacheValue> for Value {
    fn eq(&self, other: &&CacheValue) -> bool {
        *self == other.0
    }
}

impl Serialize for CacheValue {
    /// Serializes using the denormalized cache shape (untagged JSON, most specific variants first).
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        value_to_repr(&self.0).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CacheValue {
    /// Deserializes the cache shape and builds a validated [`Value`].
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let repr = CacheValueRepr::deserialize(deserializer)?;
        let value = repr_to_value(repr).map_err(de::Error::custom)?;
        Ok(Self(value))
    }
}

/// Serialized cache value representation (untagged JSON order: most specific first).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
enum CacheValueRepr {
    IntervalWithUnit { value: Interval, unit: Unit },
    NumberWithUnit { value: f64, unit: Unit },
    Interval(Interval),
    Bool(bool),
    String(String),
    Number(f64),
}

/// A numeric interval without units.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Interval {
    /// Lower bound (inclusive).
    pub min: f64,
    /// Upper bound (inclusive).
    pub max: f64,
}

/// Physical dimensions as exponents on base SI symbols (e.g. `m`, `kg`, `s`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Unit {
    /// Dimension exponents keyed by base unit symbol.
    pub dimensions: BTreeMap<String, f64>,
    /// Overall scale factor relative to the dimensioned SI form.
    pub magnitude: f64,
    /// Whether this quantity uses decibel representation.
    pub is_db: bool,
    /// Human-readable unit string for display.
    pub display_unit: String,
}

/// Failure converting serialized cache value into [`Value`].
#[derive(Debug, Clone, PartialEq)]
pub enum CacheValueConversionError {
    /// A dimension key in a cache [`Unit`] is not a recognized SI-style symbol.
    UnknownDimension {
        /// The key from the serialized cache map.
        key: String,
    },
    /// Interval bounds are not valid for [`oneil_output::Interval::new`].
    InvalidInterval {
        /// Lower bound from the cache.
        min: f64,
        /// Upper bound from the cache.
        max: f64,
    },
}

impl fmt::Display for CacheValueConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownDimension { key } => {
                write!(f, "unknown dimension key in cache unit: {key}")
            }
            Self::InvalidInterval { min, max } => write!(
                f,
                "invalid interval bounds for Oneil interval: min={min}, max={max}"
            ),
        }
    }
}

impl std::error::Error for CacheValueConversionError {}

/// Maps a runtime [`Value`] to the denormalized JSON representation.
fn value_to_repr(value: &Value) -> CacheValueRepr {
    match value {
        Value::Boolean(b) => CacheValueRepr::Bool(*b),
        Value::String(s) => CacheValueRepr::String(s.clone()),
        Value::Number(n) => number_to_repr(*n),
        Value::MeasuredNumber(m) => {
            let (n, u) = m.clone().into_number_and_unit();
            measured_parts_to_repr(n, &u)
        }
    }
}

/// Parses denormalized cache JSON into a [`Value`], validating units and intervals.
fn repr_to_value(cv: CacheValueRepr) -> Result<Value, CacheValueConversionError> {
    Ok(match cv {
        CacheValueRepr::Bool(b) => Value::Boolean(b),
        CacheValueRepr::String(s) => Value::String(s),
        CacheValueRepr::Number(x) => Value::Number(Number::Scalar(x)),
        CacheValueRepr::Interval(iv) => {
            Value::Number(Number::Interval(cache_interval_to_output(&iv)?))
        }
        CacheValueRepr::NumberWithUnit { value, unit } => {
            let u = cache_unit_to_output(&unit)?;
            Value::MeasuredNumber(MeasuredNumber::from_number_and_unit(
                Number::Scalar(value),
                u,
            ))
        }
        CacheValueRepr::IntervalWithUnit { value, unit } => {
            let u = cache_unit_to_output(&unit)?;
            Value::MeasuredNumber(MeasuredNumber::from_number_and_unit(
                Number::Interval(cache_interval_to_output(&value)?),
                u,
            ))
        }
    })
}

/// Converts measured number parts into wire representation.
fn measured_parts_to_repr(n: Number, u: &OutputUnit) -> CacheValueRepr {
    let unit = output_unit_to_cache(u);
    match n {
        Number::Scalar(value) => CacheValueRepr::NumberWithUnit { value, unit },
        Number::Interval(iv) => CacheValueRepr::IntervalWithUnit {
            value: output_interval_to_cache(iv),
            unit,
        },
    }
}

/// Converts a unitless [`Number`] into wire representation.
const fn number_to_repr(n: Number) -> CacheValueRepr {
    match n {
        Number::Scalar(x) => CacheValueRepr::Number(x),
        Number::Interval(iv) => CacheValueRepr::Interval(output_interval_to_cache(iv)),
    }
}

/// Serializes an output [`OutputUnit`] into cache [`Unit`] form.
fn output_unit_to_cache(unit: &OutputUnit) -> Unit {
    let mut dimensions = BTreeMap::new();
    for (dim, exp) in unit.dimension_map.as_map() {
        dimensions.insert(dimension_to_cache_key(*dim).to_string(), *exp);
    }
    let display_unit = format!("{}", unit.display_unit);
    Unit {
        dimensions,
        magnitude: unit.magnitude,
        is_db: unit.is_db,
        display_unit,
    }
}

/// Restores an [`OutputUnit`] from cache JSON, or errors on unknown dimension keys.
fn cache_unit_to_output(unit: &Unit) -> Result<OutputUnit, CacheValueConversionError> {
    let mut map = IndexMap::new();
    for (key, exp) in &unit.dimensions {
        let dim = cache_key_to_dimension(key)?;
        map.insert(dim, *exp);
    }
    Ok(OutputUnit {
        dimension_map: DimensionMap::new(map),
        magnitude: unit.magnitude,
        is_db: unit.is_db,
        display_unit: DisplayUnit::Unit {
            name: unit.display_unit.clone(),
            exponent: 1.0,
        },
    })
}

/// Converts cache [`Interval`] JSON into a validated output interval.
fn cache_interval_to_output(
    iv: &Interval,
) -> Result<oneil_output::Interval, CacheValueConversionError> {
    if iv.min.is_nan() && iv.max.is_nan() {
        return Ok(oneil_output::Interval::empty());
    }
    if iv.min.is_nan() || iv.max.is_nan() || iv.min > iv.max {
        return Err(CacheValueConversionError::InvalidInterval {
            min: iv.min,
            max: iv.max,
        });
    }
    Ok(oneil_output::Interval::new(iv.min, iv.max))
}

/// Converts an output interval into cache [`Interval`] form.
const fn output_interval_to_cache(iv: oneil_output::Interval) -> Interval {
    Interval {
        min: iv.min(),
        max: iv.max(),
    }
}

/// Returns the JSON object key used for a [`Dimension`] exponent map.
const fn dimension_to_cache_key(dim: Dimension) -> &'static str {
    match dim {
        Dimension::Mass => "kg",
        Dimension::Distance => "m",
        Dimension::Time => "s",
        Dimension::Temperature => "K",
        Dimension::Current => "A",
        Dimension::Information => "bit",
        Dimension::Currency => "$",
        Dimension::Substance => "mol",
        Dimension::LuminousIntensity => "cd",
    }
}

/// Parses a cache dimension key, or returns [`CacheValueConversionError::UnknownDimension`].
fn cache_key_to_dimension(key: &str) -> Result<Dimension, CacheValueConversionError> {
    match key {
        "kg" => Ok(Dimension::Mass),
        "m" => Ok(Dimension::Distance),
        "s" => Ok(Dimension::Time),
        "K" => Ok(Dimension::Temperature),
        "A" => Ok(Dimension::Current),
        "bit" => Ok(Dimension::Information),
        "$" => Ok(Dimension::Currency),
        "mol" => Ok(Dimension::Substance),
        "cd" => Ok(Dimension::LuminousIntensity),
        _ => Err(CacheValueConversionError::UnknownDimension {
            key: key.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Builds a cache unit matching common SI usage in tests.
    fn sample_cache_unit() -> Unit {
        Unit {
            dimensions: BTreeMap::from([
                ("kg".to_string(), 1.0),
                ("m".to_string(), 2.0),
                ("s".to_string(), -3.0),
            ]),
            magnitude: 1.0,
            is_db: false,
            display_unit: "W".to_string(),
        }
    }

    #[test]
    fn bool_round_trip() {
        let v = Value::Boolean(true);
        let c = CacheValue::from(v.clone());
        let back = Value::from(c);
        assert_eq!(back, v);
    }

    #[test]
    fn string_round_trip() {
        let v = Value::String("array".into());
        let c = CacheValue::from(v.clone());
        let back = Value::from(c);
        assert_eq!(back, v);
    }

    #[test]
    fn scalar_round_trip() {
        let v = Value::Number(Number::Scalar(10.0));
        let c = CacheValue::from(v.clone());
        let back = Value::from(c);
        assert_eq!(back, v);
    }

    #[test]
    fn interval_round_trip() {
        let v = Value::Number(Number::new_interval(1.0, 2.0));
        let c = CacheValue::from(v.clone());
        let back = Value::from(c);
        assert_eq!(back, v);
    }

    #[test]
    fn measured_scalar_round_trip() {
        let u = cache_unit_to_output(&sample_cache_unit()).expect("unit");
        let v = Value::MeasuredNumber(MeasuredNumber::from_number_and_unit(
            Number::Scalar(100.0),
            u,
        ));
        let c = CacheValue::from(v.clone());
        let back = Value::from(c);
        assert_eq!(back, v);
    }

    #[test]
    fn measured_interval_round_trip() {
        let u = cache_unit_to_output(&sample_cache_unit()).expect("unit");
        let v = Value::MeasuredNumber(MeasuredNumber::from_number_and_unit(
            Number::Interval(oneil_output::Interval::new(1.0, 2.0)),
            u,
        ));
        let c = CacheValue::from(v.clone());
        let back = Value::from(c);
        assert_eq!(back, v);
    }

    #[test]
    fn unknown_dimension_rejected_on_deserialize() {
        let j = json!({ "value": 1.0, "unit": {
            "dimensions": { "parsec": 1.0 },
            "magnitude": 1.0,
            "is_db": false,
            "display_unit": "pc"
        }});
        let err: Result<CacheValue, _> = serde_json::from_value(j);
        let err = err.expect_err("unknown dimension");
        assert!(err.to_string().contains("parsec"));
    }

    #[test]
    fn serde_json_round_trip_preserves_value() {
        let v = Value::Number(Number::Scalar(std::f64::consts::PI));
        let c = CacheValue::from(v.clone());
        let json = serde_json::to_string(&c).expect("serialize");
        let back: CacheValue = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.0, v);
    }

    #[test]
    fn cache_value_partial_eq_value_symmetric() {
        let v = Value::Boolean(true);
        let c = CacheValue(Value::Boolean(true));
        assert_eq!(c, v);
        assert_eq!(v, c);
        assert_ne!(CacheValue(Value::Boolean(false)), v);
    }

    #[test]
    fn cache_value_partial_eq_value_mismatched_types() {
        assert_ne!(
            CacheValue(Value::Boolean(true)),
            Value::Number(Number::Scalar(1.0))
        );
        assert_ne!(Value::String("x".into()), CacheValue(Value::Boolean(false)));
    }
}
