//! Output types for analysis (dependency graph, trees, errors).

pub mod dependency;
pub mod error;

mod tree;

pub use dependency::{DependencyGraph, DependencyTreeValue, ReferenceTreeValue};
pub use tree::Tree;
