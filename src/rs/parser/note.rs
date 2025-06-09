//! Parser for notes in an Oneil program
//!
//! Notes are used to add additional information to a parameter or test.

use nom::Parser;
use nom::combinator::all_consuming;

use super::token::note::{multi_line_note, single_line_note};
use super::util::{Result, Span};
use crate::ast::note::Note;

/// Parse a note, which can be either a single-line note starting with `~`
/// or a multi-line note delimited by `~~~`.
///
/// This function **may not consume the complete input**.
///
/// # Examples
///
/// ```
/// use oneil::parser::note::parse;
/// use oneil::parser::{Config, Span};
/// use oneil::ast::note::Note;
///
/// // Parse a single-line note
/// let input = Span::new_extra("~ This is a note\n", Config::default());
/// let (rest, note) = parse(input).unwrap();
/// assert_eq!(note, Note("This is a note".to_string()));
/// assert_eq!(rest.fragment(), &"");
/// ```
///
/// ```
/// use oneil::parser::note::parse;
/// use oneil::parser::{Config, Span};
/// use oneil::ast::note::Note;
///
/// // Parse a multi-line note with remaining input
/// let input = Span::new_extra("~~~\nLine 1\nLine 2\n~~~\nrest", Config::default());
/// let (rest, note) = parse(input).unwrap();
/// assert_eq!(note, Note("Line 1\nLine 2".to_string()));
/// assert_eq!(rest.fragment(), &"rest");
/// ```
pub fn parse(input: Span) -> Result<Note> {
    note(input)
}

/// Parse a note
///
/// This function **fails if the complete input is not consumed**.
///
/// # Examples
///
/// ```
/// use oneil::parser::note::parse_complete;
/// use oneil::parser::{Config, Span};
/// use oneil::ast::note::Note;
///
/// let input = Span::new_extra("~ This is a note\n", Config::default());
/// let (rest, note) = parse_complete(input).unwrap();
/// assert_eq!(note, Note("This is a note".to_string()));
/// assert_eq!(rest.fragment(), &"");
/// ```
///
/// ```
/// use oneil::parser::note::parse_complete;
/// use oneil::parser::{Config, Span};
///
/// let input = Span::new_extra("~ This is a note\nrest", Config::default());
/// let result = parse_complete(input);
/// assert_eq!(result.is_err(), true);
/// ```
pub fn parse_complete(input: Span) -> Result<Note> {
    all_consuming(note).parse(input)
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
        let (rest, note) = parse(input).expect("should parse single line note");
        assert_eq!(note, Note("This is a note".to_string()));
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_single_line_note_at_eof() {
        let input = Span::new_extra("~ note", Config::default());
        let (rest, note) = parse(input).expect("should parse single line note at EOF");
        assert_eq!(note, Note("note".to_string()));
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_multi_line_note() {
        let input = Span::new_extra("~~~\nLine 1\nLine 2\n~~~\nrest", Config::default());
        let (rest, note) = parse(input).expect("should parse multi-line note");
        assert_eq!(note, Note("Line 1\nLine 2".to_string()));
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_multi_line_note_extra_tildes() {
        let input = Span::new_extra("~~~~~\nfoo\nbar\n~~~~~\nrest", Config::default());
        let (rest, note) = parse(input).expect("should parse multi-line note with extra tildes");
        assert_eq!(note, Note("foo\nbar".to_string()));
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_multi_line_note_empty() {
        let input = Span::new_extra("~~~\n~~~\nrest", Config::default());
        let (rest, note) = parse(input).expect("should parse empty multi-line note");
        assert_eq!(note, Note("".to_string()));
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_multi_line_note_unclosed() {
        let input = Span::new_extra("~~~\nUnclosed note\n", Config::default());
        let result = parse(input);
        assert!(result.is_err(), "should not parse unclosed multi-line note");
    }

    #[test]
    fn test_invalid_note() {
        let input = Span::new_extra("not a note", Config::default());
        let result = parse(input);
        assert!(result.is_err(), "should not parse invalid note");
    }

    #[test]
    fn test_parse_complete_single_line_success() {
        let input = Span::new_extra("~ This is a note\n", Config::default());
        let (rest, note) = parse_complete(input).unwrap();
        assert_eq!(note, Note("This is a note".to_string()));
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_parse_complete_multi_line_success() {
        let input = Span::new_extra("~~~\nLine 1\nLine 2\n~~~\n", Config::default());
        let (rest, note) = parse_complete(input).unwrap();
        assert_eq!(note, Note("Line 1\nLine 2".to_string()));
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_parse_complete_with_remaining_input() {
        let input = Span::new_extra("~ This is a note\nrest", Config::default());
        let result = parse_complete(input);
        assert!(result.is_err());
    }
}
