#![cfg_attr(doc, doc = include_str!("../README.md"))]
//! Parser for the Oneil programming language

use oneil_ast::{
    DeclNode, ExprNode, Model, ModelNode, NoteNode, ParameterNode, TestNode, UnitExprNode,
};

mod config;
pub mod error;
mod token;

mod util;
use util::{InputSpan, Result as InternalResult};

mod declaration;
mod expression;
mod model;
mod note;
mod parameter;
mod test;
mod unit;

pub use config::Config;

/// Parses a complete Oneil model from source code.
///
/// # Errors
///
/// Returns an error if the input contains syntax errors, is incomplete, or tokenization fails.
pub fn parse_model(
    input: &str,
    config: Option<Config>,
) -> Result<ModelNode, error::ErrorsWithPartialResult<Box<Model>, error::ParserError>> {
    parse(input, config, model::parse_complete)
}

/// Parses a single declaration from source code.
///
/// # Errors
///
/// Returns an error if the input is not a valid declaration or contains syntax errors.
pub fn parse_declaration(
    input: &str,
    config: Option<Config>,
) -> Result<DeclNode, error::ParserError> {
    parse(input, config, declaration::parse_complete)
}

/// Parses a mathematical expression from source code.
///
/// # Errors
///
/// Returns an error if the input is not a valid expression or contains syntax errors.
pub fn parse_expression(
    input: &str,
    config: Option<Config>,
) -> Result<ExprNode, error::ParserError> {
    parse(input, config, expression::parse_complete)
}

/// Parses a note from source code.
///
/// # Errors
///
/// Returns an error if the input is not a valid note or contains syntax errors.
pub fn parse_note(input: &str, config: Option<Config>) -> Result<NoteNode, error::ParserError> {
    parse(input, config, note::parse_complete)
}

/// Parses a parameter definition from source code.
///
/// # Errors
///
/// Returns an error if the input is not a valid parameter definition or contains syntax errors.
pub fn parse_parameter(
    input: &str,
    config: Option<Config>,
) -> Result<ParameterNode, error::ParserError> {
    parse(input, config, parameter::parse_complete)
}

/// Parses a test definition from source code.
///
/// # Errors
///
/// Returns an error if the input is not a valid test definition or contains syntax errors.
pub fn parse_test(input: &str, config: Option<Config>) -> Result<TestNode, error::ParserError> {
    parse(input, config, test::parse_complete)
}

/// Parses a unit expression from source code.
///
/// # Errors
///
/// Returns an error if the input is not a valid unit expression or contains syntax errors.
pub fn parse_unit(input: &str, config: Option<Config>) -> Result<UnitExprNode, error::ParserError> {
    parse(input, config, unit::parse_complete)
}

/// Internal parsing function that handles the common parsing logic.
fn parse<T, E>(
    input: &str,
    config: Option<Config>,
    parser: impl Fn(InputSpan<'_>) -> InternalResult<'_, T, E>,
) -> Result<T, E> {
    let config = config.unwrap_or_default();
    let input = InputSpan::new_extra(input, config);
    let result = parser(input);

    match result {
        Ok((_rest, ast)) => Ok(ast),
        Err(nom::Err::Incomplete(_needed)) => unreachable!(
            "This should never happen because we use `complete` combinators rather than `stream` combinators"
        ),
        Err(nom::Err::Error(e) | nom::Err::Failure(e)) => Err(e),
    }
}
