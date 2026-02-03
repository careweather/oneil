#![cfg_attr(doc, doc = include_str!("../README.md"))]
//! Runtime for the Oneil programming language

#![allow(clippy::pedantic)]
// TODO: remove this once the code is cleaned up

mod cache;
mod error;
mod runtime;
mod std_builtin;

/// Re-exports for tools that are useful for debugging
/// the runtime.
pub mod data {
    pub use oneil_ast as ast;
    pub use oneil_ir as ir;
}

pub use runtime::Runtime;
