//! Error handling for the Oneil language parser.
//!
//! This module provides a comprehensive error handling system for the parser,
//! including:
//!
//! - A trait for consistent error handling across parser components
//! - Error types that capture both the type of error and its location
//! - Conversion functions between different error types
//!
//! The error system is built on top of nom's error handling, extending it with
//! Oneil-specific error types and location tracking.
//!
//! # Error Handling Strategy
//!
//! The parser uses a two-level error handling approach:
//!
//! 1. Token-level errors (`TokenError`): For low-level parsing issues like
//!    invalid characters or unterminated strings
//! 2. Parser-level errors (`ParserError`): For higher-level issues like
//!    invalid syntax or unexpected tokens

use nom::error::{FromExternalError, ParseError};

use super::{
    Span,
    token::error::{TokenError, TokenErrorKind},
};

mod parser_trait;
pub use parser_trait::ErrorHandlingParser;

pub mod partial;

/// An error that occurred during parsing.
///
/// This type represents high-level parsing errors, containing both the specific
/// kind of error and the location where it occurred. It is used for errors that
/// occur during the parsing of language constructs like declarations, expressions,
/// and parameters.
///
/// # Examples
///
/// ```
/// use oneil::parser::error::{ParserError, ParserErrorKind};
/// use oneil::parser::{Config, Span};
///
/// // Create an error for an invalid expression
/// let span = Span::new_extra("1 + ", Config::default());
/// let error = ParserError::new(ParserErrorKind::ExpectExpr, span);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ParserError {
    /// The specific kind of error that occurred
    pub kind: ParserErrorKind,
    /// The location in the source where the error occurred
    pub offset: usize,
}

impl ParserError {
    /// Creates a new parser error with the given kind and location.
    pub fn new(kind: ParserErrorKind, span: Span) -> Self {
        Self {
            kind,
            offset: span.offset(),
        }
    }
}

/// The different kinds of errors that can occur during parsing.
///
/// This enum represents all possible high-level parsing errors in the Oneil
/// language. Each variant describes a specific type of error, such as an
/// invalid declaration or an unexpected token.
///
/// # Examples
///
/// ```
/// use oneil::parser::error::ParserErrorKind;
///
/// // An error for an invalid number literal
/// let error = ParserErrorKind::InvalidNumber("123.4.5");
///
/// // An error for an invalid expression
/// let error = ParserErrorKind::ExpectExpr;
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum ParserErrorKind {
    /// Expected an AST node but found something else
    Expect(ExpectKind),
    /// Found an incomplete input
    Incomplete(IncompleteKind),
    /// Found an unexpected token
    UnexpectedToken,
    /// A token-level error occurred
    TokenError(TokenErrorKind),
    /// A low-level nom parsing error
    NomError(nom::error::ErrorKind),
}

/// The different kinds of AST nodes that can be expected
#[derive(Debug, Clone, PartialEq)]
pub enum ExpectKind {
    /// Expected a declaration
    Decl,
    /// Expected an expression
    Expr,
    /// Expected a note
    Note,
    /// Expected a parameter
    Parameter,
    /// Expected a test
    Test,
    /// Expected a unit
    Unit,
}

/// The different kinds of incomplete input that can be found
#[derive(Debug, Clone, PartialEq)]
pub enum IncompleteKind {
    /// Found an invalid `from` declaration
    FromDeclError {
        /// The offset of the `from` keyword
        from_offset: usize,
        /// The underlying error that occurred while parsing the declaration
        error: Box<ParserError>,
    },
    /// Found an invalid `import` declaration
    ImportDeclError {
        /// The offset of the `import` keyword
        import_offset: usize,
        /// The underlying error that occurred while parsing the declaration
        error: Box<ParserError>,
    },
    /// Found an invalid model input
    ModelInputError {
        /// The span containing the equals sign
        equals_offset: usize,
        /// The underlying error that occurred while parsing the input
        error: Box<ParserError>,
    },
    /// Found an invalid section label
    SectionMissingLabel {
        /// The offset of the section keyword
        section_offset: usize,
    },
    /// Found an unclosed bracket
    UnclosedBracket {
        /// The offset of the opening bracket
        bracket_left_offset: usize,
        /// The underlying error that occurred while parsing the bracketed expression
        error: Box<ParserError>,
    },
    /// Found an unclosed parenthesis
    UnclosedParen {
        /// The offset of the opening parenthesis
        paren_left_offset: usize,
        /// The underlying error that occurred while parsing the parenthesized expression
        error: Box<ParserError>,
    },
    /// Found an invalid `use` declaration
    UseDeclError {
        /// The offset of the `use` keyword
        use_offset: usize,
        /// The underlying error that occurred while parsing the declaration
        error: Box<ParserError>,
    },
    /// Found an invalid number with the given text
    InvalidNumber(String),
}

impl<'a> ParseError<Span<'a>> for ParserError {
    fn from_error_kind(input: Span<'a>, kind: nom::error::ErrorKind) -> Self {
        let kind = match kind {
            // If `all_consuming` is used, we expect the parser to consume the entire input
            nom::error::ErrorKind::Eof => ParserErrorKind::UnexpectedToken,
            _ => ParserErrorKind::NomError(kind),
        };

        Self {
            kind,
            offset: input.offset(),
        }
    }

    fn append(_input: Span<'a>, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

impl<'a> FromExternalError<Span<'a>, ParserErrorKind> for ParserError {
    fn from_external_error(
        input: Span<'a>,
        _kind: nom::error::ErrorKind,
        e: ParserErrorKind,
    ) -> Self {
        Self::new(e, input)
    }
}

/// Implements conversion from TokenError to ParserError.
///
/// This allows token-level errors to be converted into parser-level errors
/// while preserving the error information.
impl From<TokenError> for ParserError {
    fn from(e: TokenError) -> Self {
        Self {
            kind: ParserErrorKind::TokenError(e.kind),
            offset: e.offset,
        }
    }
}

/// Implements conversion from nom::error::Error to ParserError.
///
/// This allows nom's built-in errors to be converted into parser-level errors
/// while preserving the error information.
impl<'a> From<nom::error::Error<Span<'a>>> for ParserError {
    fn from(e: nom::error::Error<Span<'a>>) -> Self {
        Self {
            kind: ParserErrorKind::NomError(e.code),
            offset: e.input.offset(),
        }
    }
}
