#![cfg_attr(doc, doc = include_str!("../README.md"))]
//! Python integration for the Oneil programming language

#![allow(
    clippy::pedantic,
    reason = "this is a work in progress, remove this once it's implemented"
)]
#![allow(
    missing_docs,
    reason = "this is a work in progress, remove this once it's implemented"
)]

pub mod error;
pub mod eval;
pub mod function;
pub mod load;
mod py_compat;

pub use error::{LoadPythonImportError, PythonEvalError};
pub use eval::evaluate_python_function;
pub use function::PythonFunction;
pub use load::load_python_import;
