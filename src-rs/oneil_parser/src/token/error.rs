//! Error handling for the token parsing
//!
//! See [docs/parser/error-model.md](docs/parser/error-model.md) in the source
//! code for more information.

use nom::error::ParseError;

use super::Span;

// Re-export the ErrorHandlingParser trait from the parent module for
// convenience
pub use super::super::error::ErrorHandlingParser;

/// An error that occurred during token parsing.
///
/// Contains both the type of error and the location where it occurred.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TokenError {
    /// The specific kind of error that occurred
    pub kind: TokenErrorKind,
    /// The offset in the source where the error occurred
    pub offset: usize,
}

/// The different kinds of errors that can occur during token parsing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenErrorKind {
    /// Expected a specific token
    Expect(ExpectKind),
    /// Incomplete input
    Incomplete(IncompleteKind),
    /// A low-level nom parsing error
    NomError(nom::error::ErrorKind),
}

/// The different kinds of tokens that could have been expected.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExpectKind {
    /// Expected an end of line
    EndOfLine,
    /// Expected an identifier
    Identifier,
    /// Expected a keyword
    Keyword(ExpectKeyword),
    /// Expected a label
    Label,
    /// Expected a note
    Note,
    /// Expected a number
    Number,
    /// Expected a string
    String,
    /// Expected a symbol
    Symbol(ExpectSymbol),
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

/// The different kinds of incomplete input that could have been expected
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IncompleteKind {
    /// Unclosed note
    UnclosedNote {
        /// The offset of the note delimiter start
        delimiter_start_offset: usize,
        /// The offset of the note delimiter end
        delimiter_end_offset: usize,
    },
    /// Unclosed string
    UnclosedString {
        /// The offset of the opening quote
        open_quote_offset: usize,
    },
    /// Invalid decimal part in a number
    InvalidDecimalPart {
        /// The offset of the decimal point
        decimal_point_offset: usize,
    },
    /// Invalid exponent part in a number
    InvalidExponentPart {
        /// The offset of the exponent 'e' character
        e_offset: usize,
    },
}

impl TokenError {
    /// Creates a new TokenError
    fn new(kind: TokenErrorKind, span: Span) -> Self {
        Self {
            kind,
            offset: span.location_offset(),
        }
    }

    /// Updates the error kind
    ///
    /// This should only be happening if the error is
    /// a nom error, so it panics if it's not.
    fn update_kind(self, kind: TokenErrorKind) -> Self {
        let is_nom_error = matches!(self.kind, TokenErrorKind::NomError(_));
        assert!(
            is_nom_error,
            "Cannot update an error that is not a nom error! (attempting to update the kind {:?})",
            self.kind
        );

        Self { kind, ..self }
    }

    /// Creates a new TokenError instance for an expected end of line
    pub fn expected_end_of_line(error: Self) -> Self {
        error.update_kind(TokenErrorKind::Expect(ExpectKind::EndOfLine))
    }

    /// Creates a new TokenError instance for an expected identifier
    pub fn expected_identifier(error: Self) -> Self {
        error.update_kind(TokenErrorKind::Expect(ExpectKind::Identifier))
    }

    /// Creates a new TokenError instance for an expected keyword
    pub fn expected_keyword(keyword: ExpectKeyword) -> impl Fn(Self) -> Self {
        move |error: Self| error.update_kind(TokenErrorKind::Expect(ExpectKind::Keyword(keyword)))
    }

    /// Creates a new TokenError instance for an expected label
    pub fn expected_label(error: Self) -> Self {
        error.update_kind(TokenErrorKind::Expect(ExpectKind::Label))
    }

    /// Creates a new TokenError instance for an expected note
    pub fn expected_note(error: Self) -> Self {
        error.update_kind(TokenErrorKind::Expect(ExpectKind::Note))
    }

    /// Creates a new TokenError instance for an expected note from a span
    pub fn expected_note_from_span(span: Span) -> Self {
        Self::new(TokenErrorKind::Expect(ExpectKind::Note), span)
    }

    /// Creates a new TokenError instance for an expected number
    pub fn expected_number(error: Self) -> Self {
        error.update_kind(TokenErrorKind::Expect(ExpectKind::Number))
    }

    /// Creates a new TokenError instance for an expected string
    pub fn expected_string(error: Self) -> Self {
        error.update_kind(TokenErrorKind::Expect(ExpectKind::String))
    }

    /// Creates a new TokenError instance for an expected symbol
    pub fn expected_symbol(symbol: ExpectSymbol) -> impl Fn(Self) -> Self {
        move |error: Self| error.update_kind(TokenErrorKind::Expect(ExpectKind::Symbol(symbol)))
    }

    /// Creates a new TokenError instance for an unclosed note
    pub fn unclosed_note(delimiter_span: Span) -> impl Fn(Self) -> Self {
        move |error: Self| {
            let delimiter_start_offset = delimiter_span.location_offset();
            let delimiter_end_offset = delimiter_start_offset + delimiter_span.len();
            error.update_kind(TokenErrorKind::Incomplete(IncompleteKind::UnclosedNote {
                delimiter_start_offset,
                delimiter_end_offset,
            }))
        }
    }

    /// Creates a new TokenError instance for an unclosed string
    pub fn unclosed_string(open_quote_span: Span) -> impl Fn(Self) -> Self {
        move |error: Self| {
            let open_quote_offset = open_quote_span.location_offset();
            error.update_kind(TokenErrorKind::Incomplete(IncompleteKind::UnclosedString {
                open_quote_offset,
            }))
        }
    }

    /// Creates a new TokenError instance for an invalid decimal part in a number
    pub fn invalid_decimal_part(decimal_point_span: Span) -> impl Fn(Self) -> Self {
        move |error: Self| {
            let decimal_point_offset = decimal_point_span.location_offset();
            error.update_kind(TokenErrorKind::Incomplete(
                IncompleteKind::InvalidDecimalPart {
                    decimal_point_offset,
                },
            ))
        }
    }

    /// Creates a new TokenError instance for an invalid exponent part in a number
    pub fn invalid_exponent_part(e_span: Span) -> impl Fn(Self) -> Self {
        move |error: Self| {
            let e_offset = e_span.location_offset();
            error.update_kind(TokenErrorKind::Incomplete(
                IncompleteKind::InvalidExponentPart { e_offset },
            ))
        }
    }

    /// Checks if the error is a token error
    pub fn is_token_error(&self, kind: ExpectKeyword) -> bool {
        match self.kind {
            TokenErrorKind::Expect(ExpectKind::Keyword(keyword_kind)) => kind == keyword_kind,
            _ => false,
        }
    }

    /// Checks if the error is a symbol error
    pub fn is_symbol_error(&self, kind: ExpectSymbol) -> bool {
        match self.kind {
            TokenErrorKind::Expect(ExpectKind::Symbol(symbol_kind)) => kind == symbol_kind,
            _ => false,
        }
    }
}

impl<'a> ParseError<Span<'a>> for TokenError {
    fn from_error_kind(input: Span, kind: nom::error::ErrorKind) -> Self {
        Self {
            kind: TokenErrorKind::NomError(kind),
            offset: input.location_offset(),
        }
    }

    fn append(_input: Span, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

impl<'a> From<nom::error::Error<Span<'a>>> for TokenError {
    fn from(e: nom::error::Error<Span<'a>>) -> Self {
        Self::from_error_kind(e.input, e.code)
    }
}
