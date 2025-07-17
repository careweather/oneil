//! Parser implementation for the Oneil programming language
//!
//! This crate provides a robust and efficient parser for the Oneil language,
//! converting source code into an Abstract Syntax Tree (AST). The parser is built
//! using a recursive descent approach with the [nom] parsing library for
//! performance and reliability.
//!
//! # Architecture
//!
//! The parser is organized into several modules:
//!
//! - **`model`**: Parses complete Oneil models
//! - **`declaration`**: Handles import, from, and use declarations
//! - **`expression`**: Parses mathematical expressions and calculations
//! - **`parameter`**: Handles parameter definitions with types and values
//! - **`test`**: Parses test definitions and conditions
//! - **`unit`**: Handles unit expressions and conversions
//! - **`note`**: Parses documentation comments and notes
//! - **`token`**: Low-level tokenization and lexical analysis
//! - **`error`**: Comprehensive error handling and reporting
//!
//! # Error Handling
//!
//! The parser provides detailed error information through the `ParserError` type,
//! which includes:
//!
//! - **Location**: Precise character offset where the error occurred
//! - **Context**: Information about what was expected vs. what was found
//! - **Recovery**: Partial parsing results when possible using the `ErrorsWithPartialResult` type
//!
//! # Configuration
//!
//! The parser supports configuration options through the `Config` struct:
//!
//! # Examples
//!
//! ## Parsing a Complete Model
//!
//! ```rust
//! use oneil_parser::parse_model;
//!
//! let model_source = r#"
//! Cylinder radius: r = 5.0 :cm
//! Cylinder height: h = 10.0 :cm
//! Volume: V = pi * r^2 * h :cm^3
//! SurfaceArea: A = 2 * pi * r * (r + h) :cm^2
//! "#;
//!
//! let model = parse_model(model_source, None).unwrap();
//! ```
//!
//! ## Parsing Individual Components
//!
//! ```rust
//! use oneil_parser::{parse_expression, parse_parameter, parse_unit};
//!
//! // Parse an expression
//! let expr = parse_expression("2 * (3 + 4)", None).unwrap();
//!
//! // Parse a parameter
//! let param = parse_parameter("Radius: r = 42.0 :cm", None).unwrap();
//!
//! // Parse a unit expression
//! let unit = parse_unit("m/s^2", None).unwrap();
//! ```
//!
//! # Integration
//!
//! This parser is designed to work seamlessly with the `oneil_ast` crate,
//! providing AST nodes that can be used for further analysis, compilation,
//! or interpretation.
//!
//! ```rust
//! use oneil_parser::parse_model;
//! use oneil_ast::model::ModelNode;
//!
//! let ast: ModelNode = parse_model("Parameter radius: r = 5.0 :cm", None).unwrap();
//! // Use the AST for further processing...
//! ```
#![warn(missing_docs)]

// TODO: add tests for the errors that are produced by the parser. Right now, we
//       only test whether the parser succeeds on parsing valid input. We should
//       also add tests later to ensure that the parser produces errors when
//       parsing invalid input and that the errors are correct and clear.

use oneil_ast::{
    Model, declaration::DeclNode, expression::ExprNode, model::ModelNode, note::NoteNode,
    parameter::ParameterNode, test::TestNode, unit::UnitExprNode,
};

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

/// Parses a complete Oneil model from source code.
///
/// This function parses an entire Oneil model, including all its declarations,
/// parameters, expressions, and tests. It returns either a complete `ModelNode`
/// or detailed error information with partial results.
///
/// # Arguments
///
/// * `input` - The source code to parse
/// * `config` - Optional parser configuration (uses default if `None`)
///
/// # Returns
///
/// Returns `Ok(ModelNode)` on successful parsing, or `Err(ErrorsWithPartialResult)`
/// containing detailed error information and any partial parsing results.
///
/// # Examples
///
/// ```rust
/// use oneil_parser::parse_model;
///
/// let source = r#"
/// Cylinder radius: r = 5.0 :cm
/// Cylinder height: h = 10.0 :cm
/// Volume: V = pi * r^2 * h :cm^3
/// SurfaceArea: A = 2 * pi * r * (r + h) :cm^2
/// "#;
///
/// let model = parse_model(source, None).unwrap();
/// ```
pub fn parse_model(
    input: &str,
    config: Option<Config>,
) -> Result<ModelNode, error::ErrorsWithPartialResult<Model, error::ParserError>> {
    parse(input, config, model::parse_complete)
}

/// Parses a single declaration from source code.
///
/// Parses import, from, or use declarations. This is useful for parsing
/// individual declaration statements outside of a complete model context.
///
/// # Arguments
///
/// * `input` - The declaration source code to parse
/// * `config` - Optional parser configuration (uses default if `None`)
///
/// # Returns
///
/// Returns `Ok(DeclNode)` on successful parsing, or `Err(ParserError)` with
/// detailed error information.
///
/// # Examples
///
/// ```rust
/// use oneil_parser::parse_declaration;
///
/// // Parse an import declaration
/// let import = parse_declaration("import math", None).unwrap();
///
/// // Parse a from declaration
/// let from = parse_declaration("from constants use physics as c_p", None).unwrap();
/// ```
pub fn parse_declaration(
    input: &str,
    config: Option<Config>,
) -> Result<DeclNode, error::ParserError> {
    parse(input, config, declaration::parse_complete)
}

/// Parses a mathematical expression from source code.
///
/// Parses Oneil expressions including arithmetic operations, function calls,
/// variable references, and parenthesized expressions.
///
/// # Arguments
///
/// * `input` - The expression source code to parse
/// * `config` - Optional parser configuration (uses default if `None`)
///
/// # Returns
///
/// Returns `Ok(ExprNode)` on successful parsing, or `Err(ParserError)` with
/// detailed error information.
///
/// # Examples
///
/// ```rust
/// use oneil_parser::parse_expression;
///
/// // Parse a simple arithmetic expression
/// let expr = parse_expression("2 * (3 + 4)", None).unwrap();
///
/// // Parse a function call
/// let expr = parse_expression("sqrt(x^2 + y^2)", None).unwrap();
///
/// // Parse a variable reference
/// let expr = parse_expression("radius", None).unwrap();
/// ```
pub fn parse_expression(
    input: &str,
    config: Option<Config>,
) -> Result<ExprNode, error::ParserError> {
    parse(input, config, expression::parse_complete)
}

/// Parses a note from source code.
///
/// Parses Oneil notes which are used for documentation and comments. Notes can
/// contain LaTeX-formatted text and are preserved in the AST.
///
/// # Arguments
///
/// * `input` - The note source code to parse
/// * `config` - Optional parser configuration (uses default if `None`)
///
/// # Returns
///
/// Returns `Ok(NoteNode)` on successful parsing, or `Err(ParserError)` with
/// detailed error information.
///
/// # Examples
///
/// ```rust
/// use oneil_parser::parse_note;
///
/// // Valid note
/// let note = parse_note("~ This is a documentation note", None).unwrap();
/// ```
pub fn parse_note(input: &str, config: Option<Config>) -> Result<NoteNode, error::ParserError> {
    parse(input, config, note::parse_complete)
}

/// Parses a parameter definition from source code.
///
/// Parses Oneil parameter definitions including the parameter name, type,
/// value, and optional unit specification.
///
/// # Arguments
///
/// * `input` - The parameter source code to parse
/// * `config` - Optional parser configuration (uses default if `None`)
///
/// # Returns
///
/// Returns `Ok(ParameterNode)` on successful parsing, or `Err(ParserError)` with
/// detailed error information.
///
/// # Examples
///
/// ```rust
/// use oneil_parser::parse_parameter;
///
/// // Parse a simple parameter
/// let param = parse_parameter("Radius: r = 5.0 :cm", None).unwrap();
///
/// // Parse a parameter with units
/// let param = parse_parameter("Height: h = 10.0 :cm", None).unwrap();
/// ```
pub fn parse_parameter(
    input: &str,
    config: Option<Config>,
) -> Result<ParameterNode, error::ParserError> {
    parse(input, config, parameter::parse_complete)
}

/// Parses a test definition from source code.
///
/// Parses Oneil test definitions including test conditions and expected
/// expressions. Tests are used for validation and verification.
///
/// # Arguments
///
/// * `input` - The test source code to parse
/// * `config` - Optional parser configuration (uses default if `None`)
///
/// # Returns
///
/// Returns `Ok(TestNode)` on successful parsing, or `Err(ParserError)` with
/// detailed error information.
///
/// # Examples
///
/// ```rust
/// use oneil_parser::parse_test;
///
/// // Valid test
/// let test = parse_test("test: volume > 0", None).unwrap();
/// ```
pub fn parse_test(input: &str, config: Option<Config>) -> Result<TestNode, error::ParserError> {
    parse(input, config, test::parse_complete)
}

/// Parses a unit expression from source code.
///
/// Parses Oneil unit expressions including unit multiplication, division,
/// and exponentiation. Units are used for dimensional analysis and
/// conversion.
///
/// # Arguments
///
/// * `input` - The unit expression source code to parse
/// * `config` - Optional parser configuration (uses default if `None`)
///
/// # Returns
///
/// Returns `Ok(UnitExprNode)` on successful parsing, or `Err(ParserError)` with
/// detailed error information.
///
/// # Examples
///
/// ```rust
/// use oneil_parser::parse_unit;
///
/// // Parse a simple unit
/// assert!(parse_unit("m", None).is_ok());
///
/// // Parse a compound unit
/// assert!(parse_unit("m/s^2", None).is_ok());
///
/// // Parse a unit with parentheses
/// assert!(parse_unit("(kg * m) / s^2", None).is_ok());
/// ```
pub fn parse_unit(input: &str, config: Option<Config>) -> Result<UnitExprNode, error::ParserError> {
    parse(input, config, unit::parse_complete)
}

/// Internal parsing function that handles the common parsing logic.
///
/// This function provides the core parsing functionality used by all public
/// parsing functions. It handles configuration setup, input preparation, and
/// error conversion.
///
/// # Arguments
///
/// * `input` - The source code to parse
/// * `config` - Optional parser configuration
/// * `parser` - The specific parser function to use
///
/// # Returns
///
/// Returns the parsed result or an error, with proper error conversion from
/// nom's internal error types to the public error types.
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
