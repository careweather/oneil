#![cfg_attr(doc, doc = include_str!("../README.md"))]
//! Shared utilities for the Oneil programming language

pub mod error;
pub mod instance_path;
pub mod labels;
pub mod load_result;
pub mod partial;
pub mod paths;
pub mod search;
pub mod serde;
pub mod span;
pub mod symbols;

pub use instance_path::{EvalInstanceKey, InstancePath, RelativePath};
