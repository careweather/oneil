#![cfg_attr(doc, doc = include_str!("../README.md"))]
//! Runtime for the Oneil programming language

mod cache;
mod runtime;
mod error;
mod std_builtin;

pub mod output;

pub use runtime::Runtime;
