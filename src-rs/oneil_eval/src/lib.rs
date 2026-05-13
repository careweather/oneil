#![cfg_attr(doc, doc = include_str!("../README.md"))]
//! Evaluator for the Oneil programming language

mod context;
mod eval_expr;
mod eval_model;
mod eval_parameter;
mod eval_unit;

pub use context::{ExternalEvaluationContext, IrLoadError};
pub use eval_expr::eval_expr_in_model;
pub use eval_model::eval_model_from_graph;

// Re-export the instance graph from oneil_frontend so downstream crates don't need
// to depend on oneil_frontend directly.
pub use oneil_frontend::{ApplyDesign, InstanceGraph, InstancedModel};

#[cfg(test)]
mod test_context;

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
            use oneil_output::util::is_close;

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
    /// # use std::collections::BTreeMap;
    /// # use oneil_eval::assert_units_dimensionally_eq;
    /// # use oneil_output::{Dimension, DimensionMap};
    ///
    /// let unit = DimensionMap::new(BTreeMap::from([(Dimension::Time, 1.0)]));
    /// assert_units_dimensionally_eq!([(Dimension::Time, 1.0)], unit);
    /// ```
    macro_rules! assert_units_dimensionally_eq {
        ($expected_unit_list:expr, $actual_unit:expr) => {{
            use oneil_output::{DimensionMap, Unit};
            use std::collections::BTreeMap;

            let expected: DimensionMap = DimensionMap::new(BTreeMap::from($expected_unit_list));
            let actual: &Unit = &$actual_unit;
            assert_eq!(expected, actual.dimension_map);
        }};
    }
}
