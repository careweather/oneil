#![cfg_attr(doc, doc = include_str!("../README.md"))]
//! Runtime for the Oneil programming language

#![allow(clippy::pedantic)]
// TODO: remove this once the code is cleaned up

mod cache;
mod error;
mod runtime;

pub mod debug {
    pub use oneil_ast as ast;
    pub use oneil_ir as ir;
}
