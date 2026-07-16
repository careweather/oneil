#![cfg_attr(doc, doc = include_str!("../README.md"))]
//! Semantic analysis for the Oneil programming language.

mod context;
mod dep_graph;
mod dependency;
pub mod display;
mod independents;
pub mod output;
mod validation;

pub use context::ExternalAnalysisContext;
pub use dependency::{get_dependency_tree, get_reference_tree};
pub use independents::get_independents;
pub use validation::validate_instance_graph;
// The validation error types live next to their on-graph storage in
// `oneil_frontend`; re-exported here so existing callers (runtime, snapshot
// tests) keep importing them from `oneil_analysis`.
pub use oneil_frontend::{HostLocation, InstanceValidationError, InstanceValidationErrorKind};

#[cfg(test)]
mod test_assertions;
#[cfg(test)]
mod test_fixtures;
