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
mod test_assertions;
#[cfg(test)]
mod test_context;
#[cfg(test)]
mod test_fixtures;

#[cfg(test)]
pub use test_assertions::{
    assert_boolean, assert_invalid_type, assert_is_close, assert_is_close_case,
    assert_measured_scalar, assert_measured_scalar_case, assert_param_measured_scalar,
    assert_param_measured_scalar_case, assert_param_scalar_close, assert_param_scalar_close_case,
    assert_scalar_close, assert_scalar_close_case, assert_type_mismatch, assert_unit_eq,
    assert_unit_eq_case, assert_units_dimensionally_eq, assert_units_dimensionally_eq_case,
};
