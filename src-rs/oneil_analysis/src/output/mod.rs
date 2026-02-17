//! Output types for analysis (dependency graph, trees, errors).

pub mod error;
mod independents;
mod tree;

pub use independents::Independents;
pub use tree::{DependencyTreeValue, ReferenceTreeValue, Tree};
