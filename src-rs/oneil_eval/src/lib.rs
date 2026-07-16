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
    Assertion, check_boolean, check_invalid_type, check_is_close, check_measured_scalar,
    check_param_measured_scalar, check_param_scalar_close, check_scalar_close, check_type_mismatch,
    check_unit_eq, check_units_dimensionally_eq,
};
