//! Parser implementation for the Oneil language
//!
//! This module provides the parsing functionality for converting Oneil source
//! code into an Abstract Syntax Tree (AST).

// TODO: refactor the output to be use traits rather than a concrete type

// TODO: add tests for the errors that are produced by the parser. Right now, we
//       only test whether the parser succeeds on parsing valid input. We should
//       also add tests later to ensure that the parser produces errors when
//       parsing invalid input and that the errors are correct and clear.

use oneil_ast as ast;

mod config;
pub mod error;
mod token;

mod util;
use util::{Result as InternalResult, Span};

mod declaration;
mod expression;
mod model;
mod note;
mod parameter;
mod test;
mod unit;

pub use config::Config;

pub fn parse_model(
    input: &str,
    config: Option<Config>,
) -> Result<ast::Model, error::ErrorsWithPartialResult<ast::Model, error::ParserError>> {
    parse(input, config, model::parse_complete)
}

pub fn parse_declaration(
    input: &str,
    config: Option<Config>,
) -> Result<ast::Decl, error::ParserError> {
    parse(input, config, declaration::parse_complete)
}

pub fn parse_expression(
    input: &str,
    config: Option<Config>,
) -> Result<ast::Expr, error::ParserError> {
    parse(input, config, expression::parse_complete)
}

pub fn parse_note(input: &str, config: Option<Config>) -> Result<ast::Note, error::ParserError> {
    parse(input, config, note::parse_complete)
}

pub fn parse_parameter(
    input: &str,
    config: Option<Config>,
) -> Result<ast::Parameter, error::ParserError> {
    parse(input, config, parameter::parse_complete)
}

pub fn parse_test(input: &str, config: Option<Config>) -> Result<ast::Test, error::ParserError> {
    parse(input, config, test::parse_complete)
}

pub fn parse_unit(
    input: &str,
    config: Option<Config>,
) -> Result<ast::UnitExpr, error::ParserError> {
    parse(input, config, unit::parse_complete)
}

fn parse<T, E>(
    input: &str,
    config: Option<Config>,
    parser: impl Fn(Span) -> InternalResult<T, E>,
) -> Result<T, E> {
    let config = config.unwrap_or_default();
    let input = Span::new_extra(input, config);
    let result = parser(input);

    match result {
        Ok((_rest, ast)) => Ok(ast),
        Err(nom::Err::Incomplete(_needed)) => unreachable!(
            "This should never happen because we use `complete` combinators rather than `stream` combinators"
        ),
        Err(nom::Err::Error(e)) => Err(e),
        Err(nom::Err::Failure(e)) => Err(e),
    }
}
