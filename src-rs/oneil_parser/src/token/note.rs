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
//! use oneil_parser::token::note::{note, NoteKind};
//! use oneil_parser::{Config, Span};
//!
//! // Parse a single-line note
//! let input = Span::new_extra("~ This is a note\nrest", Config::default());
//! let (rest, (matched, kind)) = note(input).unwrap();
//! assert_eq!(matched.lexeme(), "~ This is a note");
//! assert_eq!(kind, NoteKind::SingleLine);
//!
//! // Parse a multi-line note
//! let input = Span::new_extra("~~~\nLine 1\nLine 2\n~~~\nrest", Config::default());
//! let (rest, (matched, kind)) = note(input).unwrap();
//! assert_eq!(matched.lexeme(), "~~~\nLine 1\nLine 2\n~~~");
//! assert_eq!(kind, NoteKind::MultiLine);
//! ```

use nom::Parser as _;
use nom::bytes::complete::take_while;
use nom::character::complete::{char, line_ending, not_line_ending};
use nom::combinator::{consumed, cut, flat_map, recognize, verify};
use nom::multi::many0;

use crate::token::{
    Result, Span,
    error::{ErrorHandlingParser, TokenError},
    structure::end_of_line,
    util::{Token, inline_whitespace},
};

/// The kind of note that was parsed
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NoteKind {
    /// A single-line note, which starts with `~` and ends with a newline
    SingleLine,
    /// A multi-line note, which starts and ends with `~~~` and can contain
    /// multiple lines of content, each ending with a newline
    MultiLine,
}

fn end_of_line_span(input: Span) -> Result<Span, TokenError> {
    recognize(end_of_line).parse(input)
}

/// Parses a single-line note, which starts with `~` and ends with a newline.
///
/// The note can contain any characters except for a newline, and it must be
/// followed by a newline to be considered valid.
fn single_line_note(input: Span) -> Result<Token, TokenError> {
    let (rest, matched) = recognize((char('~'), not_line_ending)).parse(input)?;
    let (rest, whitespace) = end_of_line_span(rest)?;

    if multi_line_note_delimiter(matched).is_ok() {
        let error = TokenError::expected_note_from_span(input);
        return Err(nom::Err::Error(error));
    }

    Ok((rest, Token::new(matched, whitespace)))
}

fn multi_line_note_delimiter(input: Span) -> Result<Span, TokenError> {
    recognize((
        inline_whitespace,
        verify(take_while(|c: char| c == '~'), |s: &Span| s.len() >= 3),
        inline_whitespace,
    ))
    .parse(input)
}

fn multi_line_note_content(input: Span) -> Result<Span, TokenError> {
    recognize(many0(verify((not_line_ending, line_ending), |(s, _)| {
        multi_line_note_delimiter.parse(*s).is_err()
    })))
    .parse(input)
}

/// Parses a multi-line note, which starts and ends with `~~~` and can contain
/// multiple lines of content, each ending with a newline.
///
/// The content must not contain the multi-line note delimiter `~~~` on its own
/// line, and the note must be closed with a matching `~~~` delimiter.
///
/// If the multi-line note is not closed properly, this parser will fail.
fn multi_line_note(input: Span) -> Result<Token, TokenError> {
    flat_map(
        consumed(|input| {
            let (rest, delimiter_span) = multi_line_note_delimiter.parse(input)?;
            let (rest, _) = cut(|input| -> Result<_, TokenError> {
                let (rest, _) = line_ending.parse(input)?;
                let (rest, _) = multi_line_note_content.parse(rest)?;
                let (rest, _) = multi_line_note_delimiter.parse(rest)?;
                Ok((rest, ()))
            })
            .map_failure(TokenError::unclosed_note(delimiter_span))
            .parse(rest)?;
            Ok((rest, delimiter_span))
        }),
        |(content, delimiter_span)| {
            move |input| {
                let (rest, whitespace) = cut(end_of_line_span)
                    .map_failure(TokenError::unclosed_note(delimiter_span))
                    .parse(input)?;

                Ok((rest, Token::new(content, whitespace)))
            }
        },
    )
    .parse(input)
}

/// Parses a note, which can be either a single-line or multi-line note.
///
/// If the note is not closed properly, this parser will fail.
///
/// In addition to the `Token`, this parser will also return the kind of note
/// that was parsed.
pub fn note(input: Span) -> Result<(Token, NoteKind), TokenError> {
    let single_line_note = single_line_note.map(|token| (token, NoteKind::SingleLine));
    let multi_line_note = multi_line_note.map(|token| (token, NoteKind::MultiLine));
    let note = single_line_note.or(multi_line_note);
    note.map_error(TokenError::expected_note).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;

    #[test]
    fn test_single_line_note_with_newline() {
        let input = Span::new_extra("~ this is a note\nrest", Config::default());
        let (rest, (matched, kind)) = note(input).expect("should parse single line note");
        assert_eq!(kind, NoteKind::SingleLine);
        assert_eq!(matched.lexeme(), "~ this is a note");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_single_line_note_at_eof() {
        let input = Span::new_extra("~ note", Config::default());
        let (rest, (matched, kind)) = note(input).expect("should parse single line note at EOF");
        assert_eq!(kind, NoteKind::SingleLine);
        assert_eq!(matched.lexeme(), "~ note");
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_multi_line_note_basic() {
        let input = Span::new_extra(
            "~~~\nThis is a multi-line note.\nSecond line.\n~~~\nrest",
            Config::default(),
        );
        let (rest, (matched, kind)) = note(input).expect("should parse multi-line note");
        assert_eq!(kind, NoteKind::MultiLine);
        assert!(matched.lexeme().contains("This is a multi-line note."));
        assert!(matched.lexeme().contains("Second line."));
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_multi_line_note_extra_tildes() {
        let input = Span::new_extra("~~~~~\nfoo\nbar\n~~~~~\nrest", Config::default());
        let (rest, (matched, kind)) =
            note(input).expect("should parse multi-line note with extra tildes");
        assert_eq!(kind, NoteKind::MultiLine);
        assert!(matched.lexeme().contains("foo"));
        assert!(matched.lexeme().contains("bar"));
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_multi_line_note_empty() {
        let input = Span::new_extra("~~~\n~~~\nrest", Config::default());
        let (rest, (matched, kind)) = note(input).expect("should parse empty multi-line note");
        assert_eq!(kind, NoteKind::MultiLine);
        assert_eq!(matched.lexeme(), "~~~\n~~~");
        assert_eq!(matched.whitespace(), "\n");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_multi_line_note_unclosed() {
        let input = Span::new_extra("~~~\nUnclosed note\n", Config::default());
        let res = note(input);
        assert!(res.is_err(), "should not parse unclosed multi-line note");
    }
}
