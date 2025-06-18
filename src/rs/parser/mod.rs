//! Parser implementation for the Oneil language
//!
//! This module provides the parsing functionality for converting Oneil source
//! code into an Abstract Syntax Tree (AST).

// TODO: refactor the output to be use traits rather than a concrete type

// TODO: add tests for the errors that are produced by the parser. Right now, we
//       only test whether the parser succeeds on parsing valid input. We should
//       also add tests later to ensure that the parser produces errors when
//       parsing invalid input and that the errors are correct and clear.

mod config;
pub mod error;
pub mod token;
mod util;

pub mod declaration;
pub mod expression;
pub mod model;
pub mod note;
pub mod parameter;
pub mod test;
pub mod unit;

pub use config::Config;
pub use util::{Result, Span};
