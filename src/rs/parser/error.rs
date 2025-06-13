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

use nom::{
    Parser,
    error::{FromExternalError, ParseError},
};

use super::{
    Span,
    token::error::{TokenError, TokenErrorKind},
};

/// A trait for handling parser errors in a consistent way.
///
/// This trait extends nom's `Parser` trait with additional error handling capabilities,
/// providing methods to:
///
/// - Convert between different error types
/// - Handle both recoverable (Error) and unrecoverable (Failure) errors
/// - Map errors while preserving the error type hierarchy
///
/// # Type Parameters
///
/// * `I` - The input type (usually `Span`)
/// * `O` - The output type
/// * `E` - The error type
pub trait ErrorHandlingParser<I, O, E>: Parser<I, Output = O, Error = E>
where
    E: ParseError<I>,
{
    /// Maps recoverable errors while preserving unrecoverable errors.
    ///
    /// This is useful when you want to convert only the recoverable errors
    /// to a different type, leaving the unrecoverable errors as-is. This uses `Into`
    /// to convert the errors.
    fn map_error<E2>(
        mut self,
        convert_error: impl Fn(E) -> E2,
    ) -> impl Parser<I, Output = O, Error = E2>
    where
        Self: Sized,
        E2: nom::error::ParseError<I> + From<E>,
    {
        move |input| {
            self.parse(input).map_err(|e| match e {
                nom::Err::Error(e) => nom::Err::Error(convert_error(e)),
                nom::Err::Failure(e) => nom::Err::Failure(e.into()),
                nom::Err::Incomplete(e) => nom::Err::Incomplete(e),
            })
        }
    }

    /// Maps unrecoverable errors while preserving recoverable errors.
    ///
    /// This is useful when you want to convert only the unrecoverable errors
    /// to a different type, leaving the recoverable errors as-is. This uses `Into`
    /// to convert the errors.
    fn map_failure<E2>(
        mut self,
        convert_failure: impl Fn(E) -> E2,
    ) -> impl Parser<I, Output = O, Error = E2>
    where
        Self: Sized,
        E2: nom::error::ParseError<I> + From<E>,
    {
        move |input| {
            self.parse(input).map_err(|e| match e {
                nom::Err::Error(e) => nom::Err::Error(e.into()),
                nom::Err::Failure(e) => nom::Err::Failure(convert_failure(e)),
                nom::Err::Incomplete(e) => nom::Err::Incomplete(e),
            })
        }
    }

    /// Maps both recoverable and unrecoverable errors independently.
    ///
    /// This is the most flexible error mapping function, allowing different
    /// conversions for recoverable and unrecoverable errors.
    fn map_error_and_failure<E2>(
        mut self,
        convert_error: impl Fn(E) -> E2,
        convert_failure: impl Fn(E) -> E2,
    ) -> impl Parser<I, Output = O, Error = E2>
    where
        Self: Sized,
        E2: nom::error::ParseError<I>,
    {
        move |input| {
            self.parse(input).map_err(|e| match e {
                nom::Err::Error(e) => nom::Err::Error(convert_error(e)),
                nom::Err::Failure(e) => nom::Err::Failure(convert_failure(e)),
                nom::Err::Incomplete(e) => nom::Err::Incomplete(e),
            })
        }
    }

    /// Converts errors to a new type that implements `From<E>`.
    ///
    /// This is a convenience method that uses `Into` for both recoverable and
    /// unrecoverable errors.
    fn convert_errors<E2>(self) -> impl Parser<I, Output = O, Error = E2>
    where
        Self: Sized,
        E2: nom::error::ParseError<I> + From<E>,
    {
        self.map_error_and_failure(|e| e.into(), |e| e.into())
    }
}

/// Implements the ErrorHandlingParser trait for any type that implements Parser.
///
/// This blanket implementation allows any parser to use the error handling
/// methods provided by ErrorHandlingParser.
impl<'a, I, O, E, P> ErrorHandlingParser<I, O, E> for P
where
    P: Parser<I, Output = O, Error = E>,
    E: ParseError<I>,
{
}

/// An error type representing a parsing operation that failed, but may still
/// have a partial result
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorsWithPartialResult<T, E> {
    /// The partial result of the parsing operation
    pub partial_result: T,
    /// The list of errors encountered during parsing
    pub errors: Vec<E>,
}

/// A trait for types that can be empty
///
/// This trait allows `ErrorsWithPartialResult` to be produced from
/// just an error, without a partial result, since the partial result
/// is considered to be the empty state.
pub trait CanBeEmpty {
    /// Returns an empty instance of the type
    fn empty() -> Self;
}

impl<T> CanBeEmpty for Vec<T> {
    fn empty() -> Self {
        Self::new()
    }
}

impl<T> CanBeEmpty for Option<T> {
    fn empty() -> Self {
        None
    }
}

impl<T, E> ErrorsWithPartialResult<T, E> {
    /// Creates a new `ErrorsWithPartialResult` with the given partial result
    /// and errors
    pub fn new(partial_result: T, errors: Vec<E>) -> Self {
        Self {
            partial_result: partial_result,
            errors,
        }
    }
}

impl<I, T, E> ParseError<I> for ErrorsWithPartialResult<T, E>
where
    T: CanBeEmpty,
    E: ParseError<I>,
{
    fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
        Self {
            partial_result: T::empty(),
            errors: vec![E::from_error_kind(input, kind)],
        }
    }

    fn append(_input: I, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

impl<'a, T, E> From<E> for ErrorsWithPartialResult<T, ParserError<'a>>
where
    T: CanBeEmpty,
    E: Into<ParserError<'a>>,
{
    fn from(e: E) -> Self {
        Self {
            partial_result: T::empty(),
            errors: vec![e.into()],
        }
    }
}

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
pub struct ParserError<'a> {
    /// The specific kind of error that occurred
    pub kind: ParserErrorKind<'a>,
    /// The location in the source where the error occurred
    pub span: Span<'a>,
}

impl<'a> ParserError<'a> {
    /// Creates a new parser error with the given kind and location.
    ///
    /// # Arguments
    ///
    /// * `kind` - The specific kind of error that occurred
    /// * `span` - The location in the source where the error occurred
    pub fn new(kind: ParserErrorKind<'a>, span: Span<'a>) -> Self {
        Self { kind, span }
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
pub enum ParserErrorKind<'a> {
    /// Expected a declaration but found something else
    ExpectDecl,
    /// Expected an expression but found something else
    ExpectExpr,
    /// Expected a note but found something else
    ExpectNote,
    /// Expected a parameter but found something else
    ExpectParameter,
    /// Expected a test but found something else
    ExpectTest,
    /// Expected a unit but found something else
    ExpectUnit,
    /// Found an invalid `from` declaration
    FromDeclError {
        /// The span containing the `from` keyword
        from_span: Span<'a>,
        /// The underlying error that occurred while parsing the declaration
        error: Box<ParserError<'a>>,
    },
    /// Found an invalid `import` declaration
    ImportDeclError {
        /// The span containing the `import` keyword
        import_span: Span<'a>,
        /// The underlying error that occurred while parsing the declaration
        error: Box<ParserError<'a>>,
    },
    /// Found an invalid model input
    ModelInputError {
        /// The span containing the equals sign
        equals_span: Span<'a>,
        /// The underlying error that occurred while parsing the input
        error: Box<ParserError<'a>>,
    },
    /// Found an invalid section label
    SectionMissingLabel {
        /// The span containing the section keyword
        section_span: Span<'a>,
    },
    /// Found an unclosed parenthesis
    UnclosedParen {
        /// The span containing the opening parenthesis
        paren_left_span: Span<'a>,
        /// The underlying error that occurred while parsing the parenthesized expression
        error: Box<ParserError<'a>>,
    },
    /// Found an invalid `use` declaration
    UseDeclError {
        /// The span containing the `use` keyword
        use_span: Span<'a>,
        /// The underlying error that occurred while parsing the declaration
        error: Box<ParserError<'a>>,
    },
    /// Found an invalid number with the given text
    InvalidNumber(&'a str),
    /// A token-level error occurred
    TokenError(TokenErrorKind<'a>),
    /// Found an unexpected token
    UnexpectedToken,
    /// A low-level nom parsing error
    NomError(nom::error::ErrorKind),
}

impl<'a> ParseError<Span<'a>> for ParserError<'a> {
    fn from_error_kind(input: Span<'a>, kind: nom::error::ErrorKind) -> Self {
        let kind = match kind {
            // If `all_consuming` is used, we expect the parser to consume the entire input
            nom::error::ErrorKind::Eof => ParserErrorKind::UnexpectedToken,
            _ => ParserErrorKind::NomError(kind),
        };

        Self { kind, span: input }
    }

    fn append(_input: Span<'a>, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

impl<'a> FromExternalError<Span<'a>, ParserErrorKind<'a>> for ParserError<'a> {
    fn from_external_error(
        input: Span<'a>,
        _kind: nom::error::ErrorKind,
        e: ParserErrorKind<'a>,
    ) -> Self {
        Self::new(e, input)
    }
}

/// Implements conversion from TokenError to ParserError.
///
/// This allows token-level errors to be converted into parser-level errors
/// while preserving the error information.
impl<'a> From<TokenError<'a>> for ParserError<'a> {
    fn from(e: TokenError<'a>) -> Self {
        Self {
            kind: ParserErrorKind::TokenError(e.kind),
            span: e.span,
        }
    }
}

/// Implements conversion from nom::error::Error to ParserError.
///
/// This allows nom's built-in errors to be converted into parser-level errors
/// while preserving the error information.
impl<'a> From<nom::error::Error<Span<'a>>> for ParserError<'a> {
    fn from(e: nom::error::Error<Span<'a>>) -> Self {
        Self {
            kind: ParserErrorKind::NomError(e.code),
            span: e.input,
        }
    }
}
