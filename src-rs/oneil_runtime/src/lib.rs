#![cfg_attr(doc, doc = include_str!("../README.md"))]
//! Runtime for the Oneil programming language

#![allow(clippy::pedantic)]
// TODO: remove this once the code is cleaned up

mod cache;
mod error;
mod runtime;
mod std_builtin;

pub mod output;

pub use runtime::Runtime;
