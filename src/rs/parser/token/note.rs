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
//! use oneil::parser::Span;
//!
//! // Parse a single-line note
//! let input = Span::new("~ This is a note\nrest");
//! let (rest, matched) = single_line_note(input).unwrap();
//! assert_eq!(matched.fragment(), &"~ This is a note");
//!
//! // Parse a multi-line note
//! let input = Span::new("~~~\nLine 1\nLine 2\n~~~\nrest");
//! let (rest, matched) = multi_line_note(input).unwrap();
//! assert_eq!(matched.fragment(), &"~~~\nLine 1\nLine 2\n~~~");
//! ```

use nom::Parser as _;
use nom::bytes::complete::take_while;
use nom::character::complete::{char, line_ending, not_line_ending};
use nom::combinator::{cut, recognize, verify};
use nom::multi::many0;
use nom::sequence::terminated;

use super::{Result, Span, structure::end_of_line, util::inline_whitespace};

/// Parses a single-line note, which starts with `~` and ends with a newline.
///
/// The note can contain any characters except for a newline, and it must be
/// followed by a newline to be considered valid.
pub fn single_line_note(input: Span) -> Result<Span> {
    terminated(recognize((char('~'), not_line_ending)), end_of_line).parse(input)
}

fn multi_line_note_delimiter(input: Span) -> Result<Span> {
    recognize((
        inline_whitespace,
        verify(take_while(|c: char| c == '~'), |s: &Span| s.len() >= 3),
        inline_whitespace,
    ))
    .parse(input)
}

fn multi_line_note_content(input: Span) -> Result<Span> {
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
pub fn multi_line_note(input: Span) -> Result<Span> {
    // TODO(error): add a note in the error that this failure is due to an
    //              unclosed multi-line note
    terminated(
        recognize((
            multi_line_note_delimiter,
            cut((
                line_ending,
                multi_line_note_content,
                multi_line_note_delimiter,
            )),
        )),
        end_of_line,
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_line_note_with_newline() {
        let input = Span::new("~ this is a note\nrest");
        let (rest, matched) = single_line_note(input).expect("should parse single line note");
        assert_eq!(matched.fragment(), &"~ this is a note");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_single_line_note_at_eof() {
        let input = Span::new("~ note");
        let (rest, matched) =
            single_line_note(input).expect("should parse single line note at EOF");
        assert_eq!(matched.fragment(), &"~ note");
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_multi_line_note_basic() {
        let input = Span::new("~~~\nThis is a multi-line note.\nSecond line.\n~~~\nrest");
        let (rest, matched) = multi_line_note(input).expect("should parse multi-line note");
        assert!(matched.fragment().contains("This is a multi-line note."));
        assert!(matched.fragment().contains("Second line."));
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_multi_line_note_extra_tildes() {
        let input = Span::new("~~~~~\nfoo\nbar\n~~~~~\nrest");
        let (rest, matched) =
            multi_line_note(input).expect("should parse multi-line note with extra tildes");
        assert!(matched.fragment().contains("foo"));
        assert!(matched.fragment().contains("bar"));
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_multi_line_note_empty() {
        let input = Span::new("~~~\n~~~\nrest");
        let (rest, _) = multi_line_note(input).expect("should parse empty multi-line note");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_multi_line_note_unclosed() {
        let input = Span::new("~~~\nUnclosed note\n");
        let res = multi_line_note(input);
        assert!(res.is_err(), "should not parse unclosed multi-line note");
    }
}
