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

#[cfg(feature = "rust-lib")]
pub use rust_lib::*;

#[cfg(feature = "rust-lib")]
mod rust_lib {
    pub use crate::error::{LoadPythonImportError, PythonEvalError};
    pub use crate::eval::evaluate_python_function;
    pub use crate::function::PythonFunction;
    pub use crate::load::load_python_import;
}

#[cfg(feature = "python-lib")]
pub use python_lib::*;

#[cfg(feature = "python-lib")]
mod python_lib {
    pub use crate::py_compat::oneil_py;
}
