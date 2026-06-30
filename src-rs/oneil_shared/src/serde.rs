//! Shared Serde helpers.

/// Serde helpers for `f64` values with special float support.
pub mod f64 {
    use std::fmt;

    use serde::{
        Deserializer, Serializer,
        de::{self, IgnoredAny, MapAccess, Visitor},
        ser::SerializeStruct,
    };

    const FLOAT_SPECIAL_FIELD: &str = "float_special";
    const FLOAT_SPECIAL_INFINITY: &str = "INFINITY";
    const FLOAT_SPECIAL_NEGATIVE_INFINITY: &str = "NEGATIVE_INFINITY";
    const FLOAT_SPECIAL_NAN: &str = "NAN";

    /// Serializes an `f64`, preserving special float values as objects.
    ///
    /// # Errors
    ///
    /// Returns an error if the serializer fails to serialize the number or object.
    ///
    /// # Examples
    ///
    /// ```
    /// use oneil_shared::serde::f64 as f64_serde;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Example {
    ///     #[serde(serialize_with = "f64_serde::serialize")]
    ///     value: f64,
    /// }
    ///
    /// let json = serde_json::to_value(Example { value: f64::INFINITY }).expect("serialize");
    /// assert_eq!(json, serde_json::json!({ "value": { "float_special": "INFINITY" } }));
    /// ```
    pub fn serialize<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = *value;

        let float_special_name = if value.is_nan() {
            Some(FLOAT_SPECIAL_NAN)
        } else if value == f64::INFINITY {
            Some(FLOAT_SPECIAL_INFINITY)
        } else if value == f64::NEG_INFINITY {
            Some(FLOAT_SPECIAL_NEGATIVE_INFINITY)
        } else {
            None
        };

        match float_special_name {
            Some(float_special) => {
                let mut state = serializer.serialize_struct("FloatSpecial", 1)?;
                state.serialize_field(FLOAT_SPECIAL_FIELD, float_special)?;
                state.end()
            }
            None => serializer.serialize_f64(value),
        }
    }

    /// Deserializes an `f64`, accepting normal numbers or special float objects.
    ///
    /// # Errors
    ///
    /// Returns an error if the deserialized value is not a number or a valid
    /// `float_special` object.
    ///
    /// # Examples
    ///
    /// ```
    /// use oneil_shared::serde::f64 as f64_serde;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct Example {
    ///     #[serde(deserialize_with = "f64_serde::deserialize")]
    ///     value: f64,
    /// }
    ///
    /// let value: Example = serde_json::from_value(serde_json::json!({
    ///     "value": { "float_special": "NEGATIVE_INFINITY" }
    /// }))
    /// .expect("deserialize");
    /// assert_eq!(value.value, f64::NEG_INFINITY);
    /// ```
    pub fn deserialize<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(FloatVisitor)
    }

    struct FloatVisitor;

    impl<'de> Visitor<'de> for FloatVisitor {
        type Value = f64;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a number or an object with a float_special string")
        }

        #[expect(
            clippy::cast_precision_loss,
            reason = "this matches serde's default f64 deserialization behavior for integer inputs"
        )]
        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(v as f64)
        }

        #[expect(
            clippy::cast_precision_loss,
            reason = "this matches serde's default f64 deserialization behavior for integer inputs"
        )]
        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(v as f64)
        }

        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(v)
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut float_special = None;

            while let Some(key) = map.next_key::<String>()? {
                if key == FLOAT_SPECIAL_FIELD {
                    if float_special.is_some() {
                        return Err(de::Error::duplicate_field(FLOAT_SPECIAL_FIELD));
                    }

                    float_special = Some(map.next_value::<String>()?);
                } else {
                    map.next_value::<IgnoredAny>()?;
                    return Err(de::Error::unknown_field(&key, &[FLOAT_SPECIAL_FIELD]));
                }
            }

            let float_special =
                float_special.ok_or_else(|| de::Error::missing_field(FLOAT_SPECIAL_FIELD))?;

            float_special_value(&float_special).ok_or_else(|| {
                de::Error::unknown_variant(
                    &float_special,
                    &[
                        FLOAT_SPECIAL_INFINITY,
                        FLOAT_SPECIAL_NEGATIVE_INFINITY,
                        FLOAT_SPECIAL_NAN,
                    ],
                )
            })
        }
    }

    /// Converts a special float name into its `f64` value.
    fn float_special_value(value: &str) -> Option<f64> {
        match value {
            FLOAT_SPECIAL_INFINITY => Some(f64::INFINITY),
            FLOAT_SPECIAL_NEGATIVE_INFINITY => Some(f64::NEG_INFINITY),
            FLOAT_SPECIAL_NAN => Some(f64::NAN),
            _ => None,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Example {
            #[serde(serialize_with = "serialize", deserialize_with = "deserialize")]
            value: f64,
        }

        #[test]
        fn finite_value_serializes_as_number() {
            assert_eq!(
                serde_json::to_value(Example { value: 1.5 }).expect("serialize"),
                serde_json::json!({ "value": 1.5 })
            );
        }

        #[test]
        fn infinite_value_serializes_as_special_float_object() {
            assert_eq!(
                serde_json::to_value(Example {
                    value: f64::INFINITY
                })
                .expect("serialize"),
                serde_json::json!({ "value": { "float_special": "INFINITY" } })
            );
        }

        #[test]
        fn negative_infinite_value_deserializes_from_special_float_object() {
            let example: Example = serde_json::from_value(serde_json::json!({
                "value": { "float_special": "NEGATIVE_INFINITY" }
            }))
            .expect("deserialize");

            assert_eq!(
                example,
                Example {
                    value: f64::NEG_INFINITY
                }
            );
        }

        #[test]
        fn nan_value_round_trips_through_special_float_object() {
            let json = serde_json::to_value(Example { value: f64::NAN }).expect("serialize");
            let example: Example = serde_json::from_value(json).expect("deserialize");

            assert!(example.value.is_nan());
        }
    }
}
