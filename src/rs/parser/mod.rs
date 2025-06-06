//! Parser implementation for the Oneil language
//!
//! This module provides the parsing functionality for converting Oneil source
//! code into an Abstract Syntax Tree (AST).

pub mod token;
mod util;

pub mod declaration;
pub mod expression;
pub mod model;
pub mod note;
pub mod parameter;
pub mod test;
pub mod unit;

pub use util::{Result, Span};
