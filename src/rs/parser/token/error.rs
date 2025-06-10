//! Error handling for the token parsing
//!
//! See [docs/parser/error-model.md](docs/parser/error-model.md) in the source
//! code for more information.

use nom::error::ParseError;

use super::Span;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TokenError<'a> {
    pub kind: TokenErrorKind,
    pub span: Span<'a>,
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
