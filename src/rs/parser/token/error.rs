//! Error handling for the token parsing
//!
//! See [docs/parser/error-model.md](docs/parser/error-model.md) in the source
//! code for more information.

use nom::error::ParseError;

use super::Span;

/// Converts a `nom::error::Error` to a `TokenError` with the `NomError` kind.
///
/// This is typically used with `map_err` to convert a `nom::error::Error` to a
/// `TokenError`.
///
/// Nom errors are not intended to be used directly and should at some point be
/// converted to a more specific `TokenError`.
///
/// # Examples
///
/// ```
/// use nom::{
///     character::complete::satisfy,
///     multi::many0,
///     Parser as _,
/// };
/// use oneil::parser::token::error::{self, TokenError, TokenErrorKind};
/// use oneil::parser::{util::Parser, Config, Span};
/// use nom::error::ErrorKind;
///
/// let ident_parser = many0(satisfy(|c: char| c.is_alphanumeric() || c == '_')).map_err(error::from_nom);
///
/// let input = Span::new_extra("123", Config::default());
/// let result = ident_parser.parse(input);
///
/// assert!(result.is_err());
/// assert_eq!(result.unwrap_err().kind, TokenErrorKind::NomError(ErrorKind::Char));
/// ```
pub fn from_nom<'a>(e: nom::error::Error<Span<'a>>) -> TokenError<'a> {
    TokenError::new(TokenErrorKind::NomError(e.code), e.input)
}

/// Creates a function that converts a `nom::error::Error` to a `TokenError` with
/// the given kind.
///
/// This is typically used with `map_err` to convert a `nom::error::Error` to a
/// `TokenError` with a specific kind.
///
/// # Examples
///
/// ```
/// use nom::{
///     character::complete::satisfy,
///     multi::many0,
///     Parser as _,
/// };
/// use oneil::parser::token::error::{self, TokenError, TokenErrorKind};
/// use oneil::parser::{util::Parser, Config, Span};
///
/// let ident_parser = many0(satisfy(|c: char| c.is_alphanumeric() || c == '_')).map_err(error::with_kind(TokenErrorKind::ExpectIdentifier));
///
/// let input = Span::new_extra("123", Config::default());
/// let result = ident_parser.parse(input);
///
/// assert!(result.is_err());
/// assert_eq!(result.unwrap_err().kind, TokenErrorKind::ExpectIdentifier);
/// ```
pub fn with_kind<'a>(
    kind: TokenErrorKind,
) -> impl Fn(nom::error::Error<Span<'a>>) -> TokenError<'a> {
    move |e| TokenError::new(kind, e.input)
}

/// Converts a `TokenError` to a `TokenError` with the given kind.
///
/// This is typically used with `map_err` to convert a `TokenError` to a
/// `TokenError` with a specific kind.
///
/// # Examples
///
/// ```
/// use nom::{
///     bytes::complete::tag,
///     character::complete::{satisfy, space1},
///     multi::many0,
///     sequence::cut,
///     Parser as _,
/// };
/// use oneil::parser::token::error::{self, TokenError, TokenErrorKind};
/// use oneil::parser::{util::Parser, Config, Span};
///
/// let ident_parser = many0(satisfy(|c: char| c.is_alphanumeric() || c == '_')).map_err(error::from_nom);
///
/// let import_parser = (
///     tag("import").map_err(error::from_nom),
///     cut((
///         space1.map_err(error::from_nom),
///         ident_parser,
///     )).map_err(error::convert_to_kind(TokenErrorKind::ExpectIdentifier)),
/// );
///
/// let input = Span::new_extra("import ", Config::default());
/// let result = import_parser.parse(input);
///
/// assert!(result.is_err());
/// assert_eq!(result.unwrap_err().kind, TokenErrorKind::ExpectIdentifier);
/// ```
pub fn convert_to_kind<'a>(kind: TokenErrorKind) -> impl Fn(TokenError<'a>) -> TokenError<'a> {
    move |e| TokenError::new(kind, e.span)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TokenError<'a> {
    kind: TokenErrorKind,
    span: Span<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenErrorKind {
    EndOfLine,
    ExpectLabel,
    ExpectIdentifier,
    Keyword(ExpectKeyword),
    Note(NoteError),
    Number(NumberError),
    String(StringError),
    Symbol(ExpectSymbol),
    NomError(nom::error::ErrorKind),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExpectKeyword {
    And,
    As,
    False,
    From,
    If,
    Import,
    Not,
    Or,
    Test,
    True,
    Section,
    Use,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExpectSymbol {
    BangEquals,
    Bar,
    BraceLeft,
    BraceRight,
    BracketLeft,
    BracketRight,
    Caret,
    Colon,
    Comma,
    Dollar,
    Dot,
    Equals,
    EqualsEquals,
    GreaterThan,
    GreaterThanEquals,
    LessThan,
    LessThanEquals,
    Minus,
    MinusMinus,
    ParenLeft,
    ParenRight,
    Percent,
    Plus,
    Star,
    StarStar,
    Slash,
    SlashSlash,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NoteError {
    ExpectNote,
    UnclosedNote,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumberError {
    ExpectNumber,
    InvalidDecimalPart,
    InvalidExponentPart,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StringError {
    ExpectString,
    UnterminatedString,
}

impl<'a> TokenError<'a> {
    pub fn new(kind: TokenErrorKind, span: Span<'a>) -> Self {
        Self { kind, span }
    }
}

impl<'a> ParseError<Span<'a>> for TokenError<'a> {
    fn from_error_kind(input: Span<'a>, kind: nom::error::ErrorKind) -> Self {
        Self {
            kind: TokenErrorKind::NomError(kind),
            span: input,
        }
    }

    fn append(_input: Span<'a>, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

impl<'a> From<nom::error::Error<Span<'a>>> for TokenError<'a> {
    fn from(e: nom::error::Error<Span<'a>>) -> Self {
        Self::from_error_kind(e.input, e.code)
    }
}
