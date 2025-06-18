//! Error handling for the token parsing
//!
//! See [docs/parser/error-model.md](docs/parser/error-model.md) in the source
//! code for more information.

use nom::error::ParseError;

use super::Span;

/// An error that occurred during token parsing.
///
/// Contains both the type of error and the location where it occurred.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TokenError<'a> {
    /// The specific kind of error that occurred
    pub kind: TokenErrorKind<'a>,
    /// The location in the source where the error occurred
    pub span: Span<'a>,
}

/// The different kinds of errors that can occur during token parsing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenErrorKind<'a> {
    /// Expected an end of line but found something else
    EndOfLine,
    /// Expected a label but found something else
    ExpectLabel,
    /// Expected an identifier but found something else
    ExpectIdentifier,
    /// Expected a specific keyword
    Keyword(ExpectKeyword),
    /// Error while parsing a note
    Note(NoteError<'a>),
    /// Error while parsing a number
    Number(NumberError<'a>),
    /// Error while parsing a string
    String(StringError<'a>),
    /// Expected a specific symbol
    Symbol(ExpectSymbol),
    /// A low-level nom parsing error
    NomError(nom::error::ErrorKind),
}

/// The different keywords that could have been expected.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExpectKeyword {
    /// Expected 'and' keyword
    And,
    /// Expected 'as' keyword
    As,
    /// Expected 'false' keyword
    False,
    /// Expected 'from' keyword
    From,
    /// Expected 'if' keyword
    If,
    /// Expected 'import' keyword
    Import,
    /// Expected 'not' keyword
    Not,
    /// Expected 'or' keyword
    Or,
    /// Expected 'test' keyword
    Test,
    /// Expected 'true' keyword
    True,
    /// Expected 'section' keyword
    Section,
    /// Expected 'use' keyword
    Use,
}

/// The different symbols that could have been expected.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExpectSymbol {
    /// Expected '!=' symbol
    BangEquals,
    /// Expected '|' symbol
    Bar,
    /// Expected '{' symbol
    BraceLeft,
    /// Expected '}' symbol
    BraceRight,
    /// Expected '[' symbol
    BracketLeft,
    /// Expected ']' symbol
    BracketRight,
    /// Expected '^' symbol
    Caret,
    /// Expected ':' symbol
    Colon,
    /// Expected ',' symbol
    Comma,
    /// Expected '$' symbol
    Dollar,
    /// Expected '.' symbol
    Dot,
    /// Expected '=' symbol
    Equals,
    /// Expected '==' symbol
    EqualsEquals,
    /// Expected '>' symbol
    GreaterThan,
    /// Expected '>=' symbol
    GreaterThanEquals,
    /// Expected '<' symbol
    LessThan,
    /// Expected '<=' symbol
    LessThanEquals,
    /// Expected '-' symbol
    Minus,
    /// Expected '--' symbol
    MinusMinus,
    /// Expected '(' symbol
    ParenLeft,
    /// Expected ')' symbol
    ParenRight,
    /// Expected '%' symbol
    Percent,
    /// Expected '+' symbol
    Plus,
    /// Expected '*' symbol
    Star,
    /// Expected '**' symbol
    StarStar,
    /// Expected '/' symbol
    Slash,
    /// Expected '//' symbol
    SlashSlash,
}

/// Errors that can occur while parsing notes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NoteError<'a> {
    /// Expected a note but found something else
    ExpectNote,
    /// Found an unclosed note (missing terminator)
    UnclosedNote {
        /// The span containing the beginning delimiter
        note_start_span: Span<'a>,
    },
}

/// Errors that can occur while parsing numbers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumberError<'a> {
    /// Expected a number but found something else
    ExpectNumber,
    /// Found an invalid decimal part in a number
    InvalidDecimalPart {
        /// The span containing the decimal point
        decimal_point_span: Span<'a>,
    },
    /// Found an invalid exponent part in a number
    InvalidExponentPart {
        /// The span containing the exponent 'e' character
        e_span: Span<'a>,
    },
}

/// Errors that can occur while parsing strings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StringError<'a> {
    /// Expected a string but found something else
    ExpectString,
    /// Found an unterminated string (missing closing quote)
    UnterminatedString {
        /// The span containing the opening quote
        open_quote_span: Span<'a>,
    },
}

impl<'a> TokenError<'a> {
    /// Creates a new TokenError instance.
    pub fn new(kind: TokenErrorKind<'a>, span: Span<'a>) -> Self {
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
