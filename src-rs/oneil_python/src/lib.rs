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
pub mod load;

pub use error::LoadPythonImportError;
pub use load::{load_python_import, PythonFunction};
