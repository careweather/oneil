//! Output types for the Oneil programming language.
//!
//! These types are used to represent the results of
//! evaluating Oneil models.

mod dependency;
mod model;

pub use dependency::{BuiltinDependency, DependencySet, ExternalDependency, ParameterDependency};
pub use model::{DebugInfo, Model, Parameter, PrintLevel, Test, TestResult};
