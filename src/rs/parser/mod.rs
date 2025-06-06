//! Parser implementation for the Oneil language
//!
//! This module provides the parsing functionality for converting Oneil source
//! code into an Abstract Syntax Tree (AST).

pub mod expression;
pub mod token;
pub mod unit;
mod util;

pub use util::{Result, Span};
