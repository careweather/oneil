//! Provides parsers for notes in the Oneil language.
//!
//! This module contains parsers for both single-line and multi-line notes.
//! Single-line notes start with `~` and continue until the end of the line.
//! Multi-line notes are delimited by `~~~` on their own lines and can contain
//! multiple lines of content.

use nom::Parser as _;
use nom::bytes::complete::take_while;
use nom::character::complete::{char, line_ending, not_line_ending};
use nom::combinator::{consumed, flat_map, recognize, verify};
use nom::multi::many0;

use crate::token::{
    InputSpan, Result,
    error::{ErrorHandlingParser, TokenError},
    structure::end_of_line,
    util::{Token, inline_whitespace},
};

/// The kind of note that was parsed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteKind {
    /// A single-line note, which starts with `~` and ends with a newline
    SingleLine,
    /// A multi-line note, which starts and ends with `~~~` and can contain
    /// multiple lines of content, each ending with a newline
    MultiLine,
}

/// Parses an end-of-line sequence and returns the span containing it.
///
/// This function is used internally to parse the end-of-line that follows
/// single-line notes. It recognizes the end-of-line and returns the span
/// for whitespace handling.
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns the span containing the end-of-line sequence.
fn end_of_line_span(input: InputSpan<'_>) -> Result<'_, InputSpan<'_>, TokenError> {
    recognize(end_of_line).parse(input)
}

/// Parses a single-line note, which starts with `~` and ends with a newline.
///
/// The note can contain any characters except for a newline, and it must be
/// followed by a newline to be considered valid.
///
/// This function also checks that the note is not actually a multi-line note
/// delimiter to avoid ambiguity.
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns a token containing the single-line note with its trailing whitespace.
fn single_line_note(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    let (rest, matched) = recognize((char('~'), not_line_ending)).parse(input)?;
    let (rest, whitespace) = end_of_line_span(rest)?;

    if multi_line_note_delimiter(matched).is_ok() {
        let error = TokenError::expected_note_from_span(input);
        return Err(nom::Err::Error(error));
    }

    Ok((rest, Token::new(matched, whitespace)))
}

/// Parses a multi-line note delimiter (`~~~` with optional surrounding whitespace).
///
/// This function recognizes the delimiter that starts and ends multi-line notes.
/// It requires at least 3 tilde characters (`~`) and allows optional whitespace
/// before and after the tildes.
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns the span containing the delimiter with its surrounding whitespace.
fn multi_line_note_delimiter(input: InputSpan<'_>) -> Result<'_, InputSpan<'_>, TokenError> {
    recognize((
        inline_whitespace,
        verify(take_while(|c: char| c == '~'), |s: &InputSpan<'_>| {
            s.len() >= 3
        }),
        inline_whitespace,
    ))
    .parse(input)
}

/// Parses the content of a multi-line note.
///
/// This function parses all lines of content within a multi-line note,
/// ensuring that no line contains the multi-line note delimiter `~~~`
/// on its own line (which would end the note).
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns the span containing all the note content.
fn multi_line_note_content(input: InputSpan<'_>) -> Result<'_, InputSpan<'_>, TokenError> {
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
///
/// This function uses a complex parsing strategy to handle the nested structure
/// of multi-line notes with proper error handling for unclosed notes.
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns a token containing the multi-line note with its trailing whitespace.
fn multi_line_note(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    flat_map(
        consumed(|input| {
            let (rest, delimiter_span) = multi_line_note_delimiter.parse(input)?;
            let (rest, ()) = (|input| {
                let (rest, _) = line_ending.parse(input)?;
                let (rest, _) = multi_line_note_content.parse(rest)?;
                let (rest, _) = multi_line_note_delimiter.parse(rest)?;
                Ok((rest, ()))
            })
            .or_fail_with(TokenError::unclosed_note(delimiter_span))
            .parse(rest)?;
            Ok((rest, delimiter_span))
        }),
        |(content, delimiter_span)| {
            move |input| {
                let (rest, whitespace) = end_of_line_span
                    .or_fail_with(TokenError::unclosed_note(delimiter_span))
                    .parse(input)?;

                Ok((rest, Token::new(content, whitespace)))
            }
        },
    )
    .parse(input)
}

/// Parses a note, which can be either a single-line or multi-line note.
///
/// Single-line notes start with `~` and continue until the end of the line.
/// Multi-line notes are delimited by `~~~` on their own lines and can contain
/// multiple lines of content.
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns a tuple containing:
/// - A `Token` with the complete note text including delimiters
/// - A `NoteKind` indicating whether this is a single-line or multi-line note
///
/// # Errors
///
/// Returns a `TokenError` if:
/// - The note is not properly closed (e.g. missing newline or closing delimiter)
/// - The input does not start with a valid note delimiter
///
/// # Examples
///
/// ```rust
/// use oneil_parser::parse_note;
///
/// // Single line note
/// let input = "~ This is a note\n";
/// let _ = parse_note(input, None).unwrap();
///
/// // Multi-line note
/// let input = "~~~\nThis is a\nmulti-line note\n~~~\n";
/// let _ = parse_note(input, None).unwrap();
/// ```
pub fn note(input: InputSpan<'_>) -> Result<'_, (Token<'_>, NoteKind), TokenError> {
    let single_line_note = single_line_note.map(|token| (token, NoteKind::SingleLine));
    let multi_line_note = multi_line_note.map(|token| (token, NoteKind::MultiLine));
    let note = single_line_note.or(multi_line_note);
    note.convert_error_to(TokenError::expected_note)
        .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;
    use crate::token::error::{ExpectKind, IncompleteKind, TokenErrorKind};

    mod single_line {
        use super::*;

        #[test]
        fn basic() {
            let input = InputSpan::new_extra("~ this is a note\nrest", Config::default());
            let (rest, (matched, kind)) = note(input).expect("should parse single line note");
            assert_eq!(kind, NoteKind::SingleLine);
            assert_eq!(matched.lexeme(), "~ this is a note");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn at_eof() {
            let input = InputSpan::new_extra("~ note", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse single line note at EOF");
            assert_eq!(kind, NoteKind::SingleLine);
            assert_eq!(matched.lexeme(), "~ note");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn with_trailing_whitespace() {
            let input = InputSpan::new_extra("~ note with spaces   \nrest", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse single line note with trailing whitespace");
            assert_eq!(kind, NoteKind::SingleLine);
            assert_eq!(matched.lexeme(), "~ note with spaces   ");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn with_special_characters() {
            let input = InputSpan::new_extra("~ note with @#$% symbols\nrest", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse single line note with special characters");
            assert_eq!(kind, NoteKind::SingleLine);
            assert_eq!(matched.lexeme(), "~ note with @#$% symbols");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn with_numbers() {
            let input = InputSpan::new_extra("~ note with 123 numbers\nrest", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse single line note with numbers");
            assert_eq!(kind, NoteKind::SingleLine);
            assert_eq!(matched.lexeme(), "~ note with 123 numbers");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn with_tab() {
            let input = InputSpan::new_extra("~ note\twith\ttabs\nrest", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse single line note with tabs");
            assert_eq!(kind, NoteKind::SingleLine);
            assert_eq!(matched.lexeme(), "~ note\twith\ttabs");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn empty_content() {
            let input = InputSpan::new_extra("~\nrest", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse single line note with empty content");
            assert_eq!(kind, NoteKind::SingleLine);
            assert_eq!(matched.lexeme(), "~");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn with_carriage_return() {
            let input = InputSpan::new_extra("~ note\r\nrest", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse single line note with carriage return");
            assert_eq!(kind, NoteKind::SingleLine);
            assert_eq!(matched.lexeme(), "~ note");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn empty_input() {
            let input = InputSpan::new_extra("", Config::default());
            let res = note(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(Note), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::Note)
            ));
        }

        #[test]
        fn no_tilde() {
            let input = InputSpan::new_extra("not a note\n", Config::default());
            let res = note(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(Note), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::Note)
            ));
        }

        #[test]
        fn only_tilde() {
            let input = InputSpan::new_extra("~", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse only tilde as single line note");
            assert_eq!(kind, NoteKind::SingleLine);
            assert_eq!(matched.lexeme(), "~");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn tilde_without_newline() {
            let input = InputSpan::new_extra("~ note without newline", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse tilde without newline as single line note");
            assert_eq!(kind, NoteKind::SingleLine);
            assert_eq!(matched.lexeme(), "~ note without newline");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn multi_line_delimiter_as_single_line() {
            let input = InputSpan::new_extra("~~~\nrest", Config::default());
            let res = note(input);
            let Err(nom::Err::Failure(token_error)) = res else {
                panic!("expected TokenError::Incomplete(UnclosedNote), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Incomplete(IncompleteKind::UnclosedNote { .. })
            ));
        }

        #[test]
        fn unicode_characters() {
            let input =
                InputSpan::new_extra("~ note with 世界 characters\nrest", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse single line note with unicode");
            assert_eq!(kind, NoteKind::SingleLine);
            assert_eq!(matched.lexeme(), "~ note with 世界 characters");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn emoji() {
            let input = InputSpan::new_extra("~ note with 😀 emoji\nrest", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse single line note with emoji");
            assert_eq!(kind, NoteKind::SingleLine);
            assert_eq!(matched.lexeme(), "~ note with 😀 emoji");
            assert_eq!(rest.fragment(), &"rest");
        }
    }

    mod multi_line {
        use super::*;

        #[test]
        fn basic() {
            let input = InputSpan::new_extra(
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
        fn extra_tildes() {
            let input = InputSpan::new_extra("~~~~~\nfoo\nbar\n~~~~~\nrest", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse multi-line note with extra tildes");
            assert_eq!(kind, NoteKind::MultiLine);
            assert!(matched.lexeme().contains("foo"));
            assert!(matched.lexeme().contains("bar"));
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn empty() {
            let input = InputSpan::new_extra("~~~\n~~~\nrest", Config::default());
            let (rest, (matched, kind)) = note(input).expect("should parse empty multi-line note");
            assert_eq!(kind, NoteKind::MultiLine);
            assert_eq!(matched.lexeme(), "~~~\n~~~");
            assert_eq!(matched.whitespace(), "\n");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn with_whitespace_around_delimiters() {
            let input =
                InputSpan::new_extra("   ~~~   \ncontent\n   ~~~   \nrest", Config::default());
            let (rest, (matched, kind)) = note(input)
                .expect("should parse multi-line note with whitespace around delimiters");
            assert_eq!(kind, NoteKind::MultiLine);
            assert!(matched.lexeme().contains("content"));
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn multiple_lines() {
            let input = InputSpan::new_extra(
                "~~~\nLine 1\nLine 2\nLine 3\nLine 4\n~~~\nrest",
                Config::default(),
            );
            let (rest, (matched, kind)) =
                note(input).expect("should parse multi-line note with multiple lines");
            assert_eq!(kind, NoteKind::MultiLine);
            assert!(matched.lexeme().contains("Line 1"));
            assert!(matched.lexeme().contains("Line 2"));
            assert!(matched.lexeme().contains("Line 3"));
            assert!(matched.lexeme().contains("Line 4"));
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn with_special_characters() {
            let input =
                InputSpan::new_extra("~~~\nLine with @#$% symbols\n~~~\nrest", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse multi-line note with special characters");
            assert_eq!(kind, NoteKind::MultiLine);
            assert!(matched.lexeme().contains("Line with @#$% symbols"));
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn with_numbers() {
            let input =
                InputSpan::new_extra("~~~\nLine with 123 numbers\n~~~\nrest", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse multi-line note with numbers");
            assert_eq!(kind, NoteKind::MultiLine);
            assert!(matched.lexeme().contains("Line with 123 numbers"));
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn with_tabs() {
            let input = InputSpan::new_extra("~~~\nLine\twith\ttabs\n~~~\nrest", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse multi-line note with tabs");
            assert_eq!(kind, NoteKind::MultiLine);
            assert!(matched.lexeme().contains("Line\twith\ttabs"));
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn with_carriage_returns() {
            let input =
                InputSpan::new_extra("~~~\r\nLine with CR\r\n~~~\r\nrest", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse multi-line note with carriage returns");
            assert_eq!(kind, NoteKind::MultiLine);
            assert!(matched.lexeme().contains("Line with CR"));
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn with_unicode() {
            let input = InputSpan::new_extra(
                "~~~\nLine with 世界 characters\n~~~\nrest",
                Config::default(),
            );
            let (rest, (matched, kind)) =
                note(input).expect("should parse multi-line note with unicode");
            assert_eq!(kind, NoteKind::MultiLine);
            assert!(matched.lexeme().contains("Line with 世界 characters"));
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn with_emoji() {
            let input =
                InputSpan::new_extra("~~~\nLine with 😀 emoji\n~~~\nrest", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse multi-line note with emoji");
            assert_eq!(kind, NoteKind::MultiLine);
            assert!(matched.lexeme().contains("Line with 😀 emoji"));
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn with_tilde_in_content() {
            let input = InputSpan::new_extra(
                "~~~\nLine with ~ tilde in content\n~~~\nrest",
                Config::default(),
            );
            let (rest, (matched, kind)) =
                note(input).expect("should parse multi-line note with tilde in content");
            assert_eq!(kind, NoteKind::MultiLine);
            assert!(matched.lexeme().contains("Line with ~ tilde in content"));
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn with_partial_delimiter_in_content() {
            let input = InputSpan::new_extra(
                "~~~\nLine with ~~ partial delimiter\n~~~\nrest",
                Config::default(),
            );
            let (rest, (matched, kind)) = note(input)
                .expect("should parse multi-line note with partial delimiter in content");
            assert_eq!(kind, NoteKind::MultiLine);
            assert!(matched.lexeme().contains("Line with ~~ partial delimiter"));
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn empty_input() {
            let input = InputSpan::new_extra("", Config::default());
            let res = note(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(Note), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::Note)
            ));
        }

        #[test]
        fn no_opening_delimiter() {
            let input = InputSpan::new_extra("not a note\n", Config::default());
            let res = note(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(Note), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::Note)
            ));
        }

        #[test]
        fn incomplete_delimiter() {
            let input = InputSpan::new_extra("~~\ncontent\n~~\nrest", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse incomplete delimiter as single line note");
            assert_eq!(kind, NoteKind::SingleLine);
            assert_eq!(matched.lexeme(), "~~");
            assert_eq!(rest.fragment(), &"content\n~~\nrest");
        }

        #[test]
        fn unclosed() {
            let input = InputSpan::new_extra("~~~\nUnclosed note\n", Config::default());
            let res = note(input);
            let Err(nom::Err::Failure(token_error)) = res else {
                panic!("expected TokenError::Incomplete(UnclosedNote), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Incomplete(IncompleteKind::UnclosedNote { .. })
            ));
        }

        #[test]
        fn unclosed_with_content() {
            let input = InputSpan::new_extra("~~~\nLine 1\nLine 2\n", Config::default());
            let res = note(input);
            let Err(nom::Err::Failure(token_error)) = res else {
                panic!("expected TokenError::Incomplete(UnclosedNote), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Incomplete(IncompleteKind::UnclosedNote { .. })
            ));
        }

        #[test]
        fn delimiter_in_content_line() {
            let input =
                InputSpan::new_extra("~~~\nLine 1\n~~~\nLine 2\n~~~\nrest", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse multi-line note with delimiter in content");
            assert_eq!(kind, NoteKind::MultiLine);
            assert!(matched.lexeme().contains("Line 1"));
            assert_eq!(rest.fragment(), &"Line 2\n~~~\nrest");
        }

        #[test]
        fn mismatched_delimiters() {
            let input = InputSpan::new_extra("~~~~\ncontent\n~~~\nrest", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse multi-line note with mismatched delimiters");
            assert_eq!(kind, NoteKind::MultiLine);
            assert!(matched.lexeme().contains("content"));
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn no_content_after_opening() {
            let input = InputSpan::new_extra("~~~\n", Config::default());
            let res = note(input);
            let Err(nom::Err::Failure(token_error)) = res else {
                panic!("expected TokenError::Incomplete(UnclosedNote), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Incomplete(IncompleteKind::UnclosedNote { .. })
            ));
        }
    }
}
