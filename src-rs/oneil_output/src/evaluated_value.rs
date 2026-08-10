//! JSON presentation DTO for an evaluated [`Value`].
//!
//! [`Value`]'s own `Serialize`/`Deserialize` impl is `#[serde(untagged)]` and
//! exists for a different purpose: round-tripping through
//! `oneil_py_call_cache`'s on-disk cache. External JSON consumers (the LSP's
//! rendered-view webview, `oneil test --format json`'s CI report) instead
//! want a tagged, self-describing shape that's easy to pattern-match on the
//! receiving end without re-implementing Oneil's type-coercion rules — e.g.
//! `{ "type": "number", "value": 1.0, "max": null }` rather than a bare `1.0`
//! that's ambiguous with a boolean-as-number or an interval object.
//!
//! [`EvaluatedValue`] is that tagged shape. JSON-emitting consumers share this
//! definition (and its `From<&Value>` impl); envelope types such as
//! `RenderedTree` and `TestReport` remain free to evolve independently. See
//! `docs/CODING_STANDARDS.md` (JSON Wire Types).

use oneil_shared::serde::{f64 as f64_serde, f64_option};
use serde::Serialize;

use crate::{Number, Value};

/// A JSON-tagged presentation of an evaluated [`Value`].
///
/// See the module docs for why this differs from `Value`'s own `Serialize`
/// impl.
///
/// Hand-written `serialize_with` helpers are opaque to `ts-rs`, so the float
/// fields below use `#[ts(type = ...)]` to describe
/// [`oneil_shared::serde::f64`]'s wire format (finite number or
/// `{ float_special: ... }`).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "ts-bindings", derive(ts_rs::TS))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EvaluatedValue {
    /// A boolean value.
    Boolean {
        /// The boolean value.
        value: bool,
    },
    /// A string value.
    String {
        /// The string value.
        value: String,
    },
    /// A dimensionless number (scalar or interval).
    Number {
        /// Scalar value, or interval lower bound.
        #[serde(with = "f64_serde")]
        #[cfg_attr(
            feature = "ts-bindings",
            ts(
                type = "number | { float_special: \"NAN\" | \"INFINITY\" | \"NEGATIVE_INFINITY\" }"
            )
        )]
        value: f64,
        /// Interval upper bound, `null` for scalars.
        #[serde(serialize_with = "f64_option::serialize")]
        #[cfg_attr(
            feature = "ts-bindings",
            ts(
                type = "(number | { float_special: \"NAN\" | \"INFINITY\" | \"NEGATIVE_INFINITY\" }) | null"
            )
        )]
        max: Option<f64>,
    },
    /// A number with a display unit.
    MeasuredNumber {
        /// Scalar value, or interval lower bound (in display unit).
        #[serde(with = "f64_serde")]
        #[cfg_attr(
            feature = "ts-bindings",
            ts(
                type = "number | { float_special: \"NAN\" | \"INFINITY\" | \"NEGATIVE_INFINITY\" }"
            )
        )]
        value: f64,
        /// Interval upper bound, `null` for scalars (in display unit).
        #[serde(serialize_with = "f64_option::serialize")]
        #[cfg_attr(
            feature = "ts-bindings",
            ts(
                type = "(number | { float_special: \"NAN\" | \"INFINITY\" | \"NEGATIVE_INFINITY\" }) | null"
            )
        )]
        max: Option<f64>,
        /// Display unit string, e.g. `"kg"`, `"m/s^2"`. Empty for dimensionless.
        unit: String,
    },
}

impl From<&Value> for EvaluatedValue {
    fn from(value: &Value) -> Self {
        match value {
            Value::Boolean(b) => Self::Boolean { value: *b },
            Value::String(s) => Self::String { value: s.clone() },
            Value::Number(n) => {
                let (value, max) = min_and_optional_max(n);
                Self::Number { value, max }
            }
            Value::MeasuredNumber(measured) => {
                let (number, unit) = measured.clone().into_number_and_unit();
                let (value, max) = min_and_optional_max(&number);
                Self::MeasuredNumber {
                    value,
                    max,
                    unit: unit.to_string(),
                }
            }
        }
    }
}

/// Splits a [`Number`] into its lower bound and, for intervals only, its
/// upper bound.
///
/// Scalars report `max: None` (there's nothing to distinguish from `value`);
/// intervals report `max: Some(upper_bound)`.
const fn min_and_optional_max(number: &Number) -> (f64, Option<f64>) {
    match number {
        Number::Scalar(v) => (*v, None),
        Number::Interval(interval) => (interval.min(), Some(interval.max())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Dimension, DimensionMap, DisplayUnit, Interval, MeasuredNumber, Unit};

    #[expect(
        clippy::float_cmp,
        reason = "we want to compare the exact values of the floats, which are passed through unmodified"
    )]
    #[test]
    fn min_and_optional_max_reports_scalars_without_a_max() {
        let (value, max) = min_and_optional_max(&Number::Scalar(4.0));

        assert_eq!(value, 4.0);
        assert_eq!(max, None);
    }

    #[expect(
        clippy::float_cmp,
        reason = "we want to compare the exact values of the floats, which are passed through unmodified"
    )]
    #[test]
    fn min_and_optional_max_reports_intervals_with_min_and_max() {
        let interval = Interval::new(1.0, 3.0);
        let (value, max) = min_and_optional_max(&Number::Interval(interval));

        assert_eq!(value, 1.0);
        assert_eq!(max, Some(3.0));
    }

    #[test]
    fn from_boolean_serializes_with_type_tag() {
        let value = EvaluatedValue::from(&Value::Boolean(true));
        let json = serde_json::to_value(&value).expect("serialize");

        assert_eq!(
            json,
            serde_json::json!({ "type": "boolean", "value": true })
        );
    }

    #[test]
    fn from_string_serializes_with_type_tag() {
        let value = EvaluatedValue::from(&Value::String("hello".to_string()));
        let json = serde_json::to_value(&value).expect("serialize");

        assert_eq!(
            json,
            serde_json::json!({ "type": "string", "value": "hello" })
        );
    }

    #[test]
    fn from_scalar_number_omits_max() {
        let value = EvaluatedValue::from(&Value::Number(Number::Scalar(2.5)));
        let json = serde_json::to_value(&value).expect("serialize");

        assert_eq!(
            json,
            serde_json::json!({ "type": "number", "value": 2.5, "max": null })
        );
    }

    #[test]
    fn from_interval_number_includes_max() {
        let value = EvaluatedValue::from(&Value::Number(Number::new_interval(1.0, 2.0)));
        let json = serde_json::to_value(&value).expect("serialize");

        assert_eq!(
            json,
            serde_json::json!({ "type": "number", "value": 1.0, "max": 2.0 })
        );
    }

    #[test]
    fn from_measured_number_includes_display_unit() {
        let unit = Unit {
            dimension_map: DimensionMap::new(std::collections::BTreeMap::from([(
                Dimension::Mass,
                1.0,
            )])),
            magnitude: 1.0,
            is_db: false,
            display_unit: DisplayUnit::Unit {
                name: "kg".to_string(),
                exponent: 1.0,
            },
        };
        let measured = MeasuredNumber::from_number_and_unit(Number::Scalar(10.0), unit);
        let value = EvaluatedValue::from(&Value::MeasuredNumber(measured));
        let json = serde_json::to_value(&value).expect("serialize");

        assert_eq!(
            json,
            serde_json::json!({ "type": "measured_number", "value": 10.0, "max": null, "unit": "kg" })
        );
    }
}
