//! Provides parsers for notes in the Oneil language.
//!
//! This module contains parsers for both single-line and multi-line notes.
//! Single-line notes start with `~` and continue until the end of the line.
//! Multi-line notes are delimited by `~~~` on their own lines and can contain
//! multiple lines of content.
//!
//! # Examples
//!
//! ```
//! use oneil::parser::token::note::{single_line_note, multi_line_note};
//! use oneil::parser::{Config, Span};
//!
//! // Parse a single-line note
//! let input = Span::new_extra("~ This is a note\nrest", Config::default());
//! let (rest, matched) = single_line_note(input).unwrap();
//! assert_eq!(matched.fragment(), &"~ This is a note");
//!
//! // Parse a multi-line note
//! let input = Span::new_extra("~~~\nLine 1\nLine 2\n~~~\nrest", Config::default());
//! let (rest, matched) = multi_line_note(input).unwrap();
//! assert_eq!(matched.fragment(), &"~~~\nLine 1\nLine 2\n~~~");
//! ```

use nom::Parser as _;
use nom::bytes::complete::take_while;
use nom::character::complete::{char, line_ending, not_line_ending};
use nom::combinator::{consumed, cut, flat_map, recognize, value, verify};
use nom::multi::many0;
use nom::sequence::terminated;

use super::{
    Result, Span,
    error::{NoteError, TokenError, TokenErrorKind},
    structure::end_of_line,
    util::inline_whitespace,
};
use crate::parser::error::ErrorHandlingParser as _;

/// Parses a single-line note, which starts with `~` and ends with a newline.
///
/// The note can contain any characters except for a newline, and it must be
/// followed by a newline to be considered valid.
pub fn single_line_note(input: Span) -> Result<Span, TokenError> {
    // Needed for type inference
    let recognize = recognize::<_, nom::error::Error<Span>, _>;

    // Needed for type inference
    verify(
        terminated(
            recognize((char('~'), not_line_ending)).convert_errors(),
            end_of_line,
        ),
        |span| multi_line_note_delimiter(*span).is_err(),
    )
    .map_error(|e| TokenError::new(TokenErrorKind::Note(NoteError::ExpectNote), e.span))
    .parse(input)
}

fn multi_line_note_delimiter(input: Span) -> Result<Span, TokenError> {
    // Needed for type inference
    let take_while = take_while::<_, _, nom::error::Error<Span>>;

    recognize((
        inline_whitespace,
        verify(take_while(|c: char| c == '~'), |s: &Span| s.len() >= 3).convert_errors(),
        inline_whitespace,
    ))
    .parse(input)
}

fn multi_line_note_content(input: Span) -> Result<Span, TokenError> {
    let not_line_ending = not_line_ending::<_, nom::error::Error<Span>>;

    recognize(many0(verify((not_line_ending, line_ending), |(s, _)| {
        multi_line_note_delimiter.parse(*s).is_err()
    })))
    .convert_errors()
    .parse(input)
}

/// Parses a multi-line note, which starts and ends with `~~~` and can contain
/// multiple lines of content, each ending with a newline.
///
/// The content must not contain the multi-line note delimiter `~~~` on its own
/// line, and the note must be closed with a matching `~~~` delimiter.
///
/// If the multi-line note is not closed properly, this parser will fail.
pub fn multi_line_note(input: Span) -> Result<Span, TokenError> {
    let unclosed_note_kind =
        |note_start_span| TokenErrorKind::Note(NoteError::UnclosedNote { note_start_span });
    let expect_note_kind = TokenErrorKind::Note(NoteError::ExpectNote);

    flat_map(
        consumed(flat_map(multi_line_note_delimiter, |delimiter_span| {
            value(
                delimiter_span,
                cut((
                    line_ending,
                    multi_line_note_content,
                    multi_line_note_delimiter,
                ))
                .map_failure(move |e| TokenError::new(unclosed_note_kind(delimiter_span), e.span)),
            )
        })),
        |(content, delimiter_span)| {
            value(
                content,
                cut(end_of_line).map_failure(move |e| {
                    TokenError::new(unclosed_note_kind(delimiter_span), e.span)
                }),
            )
        },
    )
    .map_error(|e| TokenError::new(expect_note_kind, e.span))
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Config;

    #[test]
    fn test_single_line_note_with_newline() {
        let input = Span::new_extra("~ this is a note\nrest", Config::default());
        let (rest, matched) = single_line_note(input).expect("should parse single line note");
        assert_eq!(matched.fragment(), &"~ this is a note");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_single_line_note_at_eof() {
        let input = Span::new_extra("~ note", Config::default());
        let (rest, matched) =
            single_line_note(input).expect("should parse single line note at EOF");
        assert_eq!(matched.fragment(), &"~ note");
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_multi_line_note_basic() {
        let input = Span::new_extra(
            "~~~\nThis is a multi-line note.\nSecond line.\n~~~\nrest",
            Config::default(),
        );
        let (rest, matched) = multi_line_note(input).expect("should parse multi-line note");
        assert!(matched.fragment().contains("This is a multi-line note."));
        assert!(matched.fragment().contains("Second line."));
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_multi_line_note_extra_tildes() {
        let input = Span::new_extra("~~~~~\nfoo\nbar\n~~~~~\nrest", Config::default());
        let (rest, matched) =
            multi_line_note(input).expect("should parse multi-line note with extra tildes");
        assert!(matched.fragment().contains("foo"));
        assert!(matched.fragment().contains("bar"));
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_multi_line_note_empty() {
        let input = Span::new_extra("~~~\n~~~\nrest", Config::default());
        let (rest, _) = multi_line_note(input).expect("should parse empty multi-line note");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_multi_line_note_unclosed() {
        let input = Span::new_extra("~~~\nUnclosed note\n", Config::default());
        let res = multi_line_note(input);
        assert!(res.is_err(), "should not parse unclosed multi-line note");
    }
}
