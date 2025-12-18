#![cfg_attr(doc, doc = include_str!("../README.md"))]
//! Evaluator for the Oneil programming language

pub mod builtin;
mod context;
mod error;
mod eval_expr;
mod eval_model;
mod eval_model_collection;
mod eval_parameter;
mod eval_unit;
pub mod result;
pub mod value;

pub use error::EvalError;
pub use eval_expr::eval_expr;
pub use eval_model::eval_model;
pub use eval_model_collection::eval_model_collection;
pub use eval_parameter::eval_parameter;
pub use eval_unit::eval_unit;

#[cfg(test)]
mod test {

    #[macro_export]
    /// Asserts that two floating point numbers are close to each other.
    ///
    /// ```rust
    /// # use oneil_eval::assert_is_close;
    /// assert_is_close!(0.1 + 0.2, 0.3);
    /// ```
    macro_rules! assert_is_close {
        ($expected:expr, $actual:expr) => {{
            use $crate::value::util::is_close;

            let expected: f64 = $expected;
            let actual: f64 = $actual;
            assert!(
                is_close(expected, actual),
                "expected: {}, actual: {}",
                expected,
                actual
            );
        }};
    }

    #[macro_export]
    /// Asserts that two units are equal.
    ///
    /// ```rust
    /// # use std::collections::HashMap;
    /// # use oneil_eval::{assert_units_eq, value::{Dimension, Unit}};
    ///
    /// let unit = Unit::new(HashMap::from([(Dimension::Time, 1.0)]));
    /// assert_units_eq!([(Dimension::Time, 1.0)], unit);
    /// ```
    macro_rules! assert_units_eq {
        ($expected_unit_list:expr, $actual_unit:expr) => {{
            use std::collections::HashMap;
            use $crate::value::Unit;

            let expected: Unit = Unit::new(HashMap::from($expected_unit_list));
            let actual: Unit = $actual_unit;
            assert_eq!(expected, actual);
        }};
    }
}
