//! Parser implementation for the Oneil language
//!
//! This module provides the parsing functionality for converting Oneil source
//! code into an Abstract Syntax Tree (AST).

// TODO: rewrite parsers to make their intent more clear.  For example, use `let
//       x = foo().parse(input)?; Ok(x + 1)` instead of `foo().map(|x| x + 1)`.

// TODO: refactor the output to be use traits rather than a concrete type

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
