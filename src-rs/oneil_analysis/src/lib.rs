#![cfg_attr(doc, doc = include_str!("../README.md"))]
//! Semantic analysis for the Oneil programming language.

mod context;
mod dep_graph;
mod dependency;
mod independents;
pub mod output;

pub use context::ExternalAnalysisContext;
pub use dependency::{get_dependency_tree, get_reference_tree};
pub use independents::get_independents;
