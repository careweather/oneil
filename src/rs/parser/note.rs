//! Parser for notes in an Oneil program
//!
//! Notes are used to add additional information to a parameter or test.

use nom::Parser;

use super::token::note::{multi_line_note, single_line_note};
use super::util::{Result, Span};
use crate::ast::note::Note;

/// Parse a note, which can be either a single-line note starting with `~`
/// or a multi-line note delimited by `~~~`.
///
/// # Examples
///
/// ```
/// use oneil::parser::note::parse;
/// use oneil::parser::{Config, Span};
/// use oneil::ast::note::Note;
///
/// // Parse a single-line note
/// let input = Span::new_extra("~ This is a note\nrest", Config::default());
/// let (_, note) = parse(input).unwrap();
/// assert_eq!(note, Note("This is a note".to_string()));
///
/// // Parse a multi-line note
/// let input = Span::new_extra("~~~\nLine 1\nLine 2\n~~~\nrest", Config::default());
/// let (_, note) = parse(input).unwrap();
/// assert_eq!(note, Note("Line 1\nLine 2".to_string()));
/// ```
pub fn parse(input: Span) -> Result<Note> {
    note(input)
}

fn note(input: Span) -> Result<Note> {
    let single_line_note = single_line_note.map(|span| {
        let content = span.trim_start_matches('~').trim();
        Note(content.to_string())
    });

    let multi_line_note = multi_line_note.map(|span| {
        let content = span.fragment().trim_matches('~').trim();
        Note(content.to_string())
    });

    single_line_note.or(multi_line_note).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Config;

    #[test]
    fn test_single_line_note() {
        let input = Span::new_extra("~ This is a note\nrest", Config::default());
        let (rest, note) = note(input).expect("should parse single line note");
        assert_eq!(note, Note("This is a note".to_string()));
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_single_line_note_at_eof() {
        let input = Span::new_extra("~ note", Config::default());
        let (rest, note) = note(input).expect("should parse single line note at EOF");
        assert_eq!(note, Note("note".to_string()));
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_multi_line_note() {
        let input = Span::new_extra("~~~\nLine 1\nLine 2\n~~~\nrest", Config::default());
        let (rest, note) = note(input).expect("should parse multi-line note");
        assert_eq!(note, Note("Line 1\nLine 2".to_string()));
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_multi_line_note_extra_tildes() {
        let input = Span::new_extra("~~~~~\nfoo\nbar\n~~~~~\nrest", Config::default());
        let (rest, note) = note(input).expect("should parse multi-line note with extra tildes");
        assert_eq!(note, Note("foo\nbar".to_string()));
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_multi_line_note_empty() {
        let input = Span::new_extra("~~~\n~~~\nrest", Config::default());
        let (rest, note) = note(input).expect("should parse empty multi-line note");
        assert_eq!(note, Note("".to_string()));
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_multi_line_note_unclosed() {
        let input = Span::new_extra("~~~\nUnclosed note\n", Config::default());
        assert!(
            note(input).is_err(),
            "should not parse unclosed multi-line note"
        );
    }

    #[test]
    fn test_invalid_note() {
        let input = Span::new_extra("not a note", Config::default());
        assert!(note(input).is_err(), "should not parse invalid note");
    }
}
