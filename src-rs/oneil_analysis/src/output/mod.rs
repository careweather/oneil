//! Output types for analysis (dependency graph, trees, errors).

pub mod error;

mod tree;

pub use tree::{DependencyTreeValue, ReferenceTreeValue, Tree};
