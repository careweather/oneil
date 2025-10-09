//! Provides parsers for notes in the Oneil language.
//!
//! This module contains parsers for both single-line and multi-line notes.
//! Single-line notes start with `~` and continue until the end of the line.
//! Multi-line notes are delimited by `~~~` on their own lines and can contain
//! multiple lines of content.

use nom::Parser as _;
use nom::bytes::complete::take_while;
use nom::character::complete::{char, line_ending, not_line_ending};
use nom::combinator::{recognize, verify};
use nom::multi::many0;
use oneil_shared::span::{SourceLocation, Span};

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
fn end_of_line_span(input: InputSpan<'_>) -> Result<'_, InputSpan<'_>, TokenError> {
    recognize(end_of_line).parse(input)
}

/// Parses a single-line note, which starts with `~` and ends with a newline.
fn single_line_note(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    let (rest, matched) = recognize((char('~'), not_line_ending)).parse(input)?;

    let lexeme_str = matched.fragment();

    let lexeme_start_line = usize::try_from(matched.location_line())
        .expect("usize should be greater than or equal to u32");
    let lexeme_start = SourceLocation {
        offset: matched.location_offset(),
        line: lexeme_start_line,
        column: matched.get_column(),
    };

    let lexeme_end_line = usize::try_from(rest.location_line())
        .expect("usize should be greater than or equal to u32");
    let lexeme_end = SourceLocation {
        offset: rest.location_offset(),
        line: lexeme_end_line,
        column: rest.get_column(),
    };

    let lexeme_span = Span::new(lexeme_start, lexeme_end);

    let (rest, whitespace) = end_of_line_span(rest)?;

    if multi_line_note_delimiter(matched).is_ok() {
        let error = TokenError::expected_note_from_span(input);
        return Err(nom::Err::Error(error));
    }

    let whitespace_start_line = usize::try_from(whitespace.location_line())
        .expect("usize should be greater than or equal to u32");
    let whitespace_start = SourceLocation {
        offset: whitespace.location_offset(),
        line: whitespace_start_line,
        column: whitespace.get_column(),
    };

    let whitespace_end_line = usize::try_from(rest.location_line())
        .expect("usize should be greater than or equal to u32");
    let whitespace_end = SourceLocation {
        offset: rest.location_offset(),
        line: whitespace_end_line,
        column: rest.get_column(),
    };

    let whitespace_span = Span::new(whitespace_start, whitespace_end);

    Ok((
        rest,
        Token {
            lexeme_str,
            lexeme_span,
            whitespace_span,
        },
    ))
}

/// Parses a multi-line note delimiter (`~~~` with optional surrounding whitespace).
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
fn multi_line_note(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    let (rest, content) = recognize(|input| {
        let (rest, delimiter_span) = multi_line_note_delimiter.parse(input)?;
        let (rest, closing_delimiter_span) = (|input| {
            let (rest, _) = line_ending.parse(input)?;
            let (rest, _) = multi_line_note_content.parse(rest)?;
            let (rest, closing_delimiter_span) = multi_line_note_delimiter.parse(rest)?;
            Ok((rest, ()))
        })
        .or_fail_with(TokenError::unclosed_note(delimiter_span))
        .parse(rest)?;
        Ok((rest, ()))
    })
    .parse(input)?;

    let lexeme_str = content.fragment();

    let lexeme_start_line = usize::try_from(content.location_line())
        .expect("usize should be greater than or equal to u32");
    let lexeme_start = SourceLocation {
        offset: content.location_offset(),
        line: lexeme_start_line,
        column: content.get_column(),
    };

    let lexeme_end_line = usize::try_from(rest.location_line())
        .expect("usize should be greater than or equal to u32");
    let lexeme_end = SourceLocation {
        offset: rest.location_offset(),
        line: lexeme_end_line,
        column: rest.get_column(),
    };

    let lexeme_span = Span::new(lexeme_start, lexeme_end);

    let (rest, whitespace) = end_of_line_span
        .or_fail_with(TokenError::invalid_closing_delimiter)
        .parse(rest)?;

    let whitespace_start_line = usize::try_from(whitespace.location_line())
        .expect("usize should be greater than or equal to u32");
    let whitespace_start = SourceLocation {
        offset: whitespace.location_offset(),
        line: whitespace_start_line,
        column: whitespace.get_column(),
    };

    let whitespace_end_line = usize::try_from(rest.location_line())
        .expect("usize should be greater than or equal to u32");
    let whitespace_end = SourceLocation {
        offset: rest.location_offset(),
        line: whitespace_end_line,
        column: rest.get_column(),
    };

    let whitespace_span = Span::new(whitespace_start, whitespace_end);

    Ok((
        rest,
        Token {
            lexeme_str,
            lexeme_span,
            whitespace_span,
        },
    ))
}

/// Parses a note, which can be either a single-line or multi-line note.
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
            assert_eq!(matched.lexeme_str, "~ this is a note");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn at_eof() {
            let input = InputSpan::new_extra("~ note", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse single line note at EOF");
            assert_eq!(kind, NoteKind::SingleLine);
            assert_eq!(matched.lexeme_str, "~ note");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn with_trailing_whitespace() {
            let input = InputSpan::new_extra("~ note with spaces   \nrest", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse single line note with trailing whitespace");
            assert_eq!(kind, NoteKind::SingleLine);
            assert_eq!(matched.lexeme_str, "~ note with spaces   ");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn with_special_characters() {
            let input = InputSpan::new_extra("~ note with @#$% symbols\nrest", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse single line note with special characters");
            assert_eq!(kind, NoteKind::SingleLine);
            assert_eq!(matched.lexeme_str, "~ note with @#$% symbols");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn with_numbers() {
            let input = InputSpan::new_extra("~ note with 123 numbers\nrest", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse single line note with numbers");
            assert_eq!(kind, NoteKind::SingleLine);
            assert_eq!(matched.lexeme_str, "~ note with 123 numbers");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn with_tab() {
            let input = InputSpan::new_extra("~ note\twith\ttabs\nrest", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse single line note with tabs");
            assert_eq!(kind, NoteKind::SingleLine);
            assert_eq!(matched.lexeme_str, "~ note\twith\ttabs");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn empty_content() {
            let input = InputSpan::new_extra("~\nrest", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse single line note with empty content");
            assert_eq!(kind, NoteKind::SingleLine);
            assert_eq!(matched.lexeme_str, "~");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn with_carriage_return() {
            let input = InputSpan::new_extra("~ note\r\nrest", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse single line note with carriage return");
            assert_eq!(kind, NoteKind::SingleLine);
            assert_eq!(matched.lexeme_str, "~ note");
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
            assert_eq!(matched.lexeme_str, "~");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn tilde_without_newline() {
            let input = InputSpan::new_extra("~ note without newline", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse tilde without newline as single line note");
            assert_eq!(kind, NoteKind::SingleLine);
            assert_eq!(matched.lexeme_str, "~ note without newline");
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
            assert_eq!(matched.lexeme_str, "~ note with 世界 characters");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn emoji() {
            let input = InputSpan::new_extra("~ note with 😀 emoji\nrest", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse single line note with emoji");
            assert_eq!(kind, NoteKind::SingleLine);
            assert_eq!(matched.lexeme_str, "~ note with 😀 emoji");
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
            assert!(matched.lexeme_str.contains("This is a multi-line note."));
            assert!(matched.lexeme_str.contains("Second line."));
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn extra_tildes() {
            let input = InputSpan::new_extra("~~~~~\nfoo\nbar\n~~~~~\nrest", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse multi-line note with extra tildes");
            assert_eq!(kind, NoteKind::MultiLine);
            assert!(matched.lexeme_str.contains("foo"));
            assert!(matched.lexeme_str.contains("bar"));
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn empty() {
            let input = InputSpan::new_extra("~~~\n~~~\nrest", Config::default());
            let (rest, (matched, kind)) = note(input).expect("should parse empty multi-line note");
            assert_eq!(kind, NoteKind::MultiLine);
            assert_eq!(matched.lexeme_str, "~~~\n~~~");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn with_whitespace_around_delimiters() {
            let input =
                InputSpan::new_extra("   ~~~   \ncontent\n   ~~~   \nrest", Config::default());
            let (rest, (matched, kind)) = note(input)
                .expect("should parse multi-line note with whitespace around delimiters");
            assert_eq!(kind, NoteKind::MultiLine);
            assert!(matched.lexeme_str.contains("content"));
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
            assert!(matched.lexeme_str.contains("Line 1"));
            assert!(matched.lexeme_str.contains("Line 2"));
            assert!(matched.lexeme_str.contains("Line 3"));
            assert!(matched.lexeme_str.contains("Line 4"));
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn with_special_characters() {
            let input =
                InputSpan::new_extra("~~~\nLine with @#$% symbols\n~~~\nrest", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse multi-line note with special characters");
            assert_eq!(kind, NoteKind::MultiLine);
            assert!(matched.lexeme_str.contains("Line with @#$% symbols"));
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn with_numbers() {
            let input =
                InputSpan::new_extra("~~~\nLine with 123 numbers\n~~~\nrest", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse multi-line note with numbers");
            assert_eq!(kind, NoteKind::MultiLine);
            assert!(matched.lexeme_str.contains("Line with 123 numbers"));
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn with_tabs() {
            let input = InputSpan::new_extra("~~~\nLine\twith\ttabs\n~~~\nrest", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse multi-line note with tabs");
            assert_eq!(kind, NoteKind::MultiLine);
            assert!(matched.lexeme_str.contains("Line\twith\ttabs"));
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn with_carriage_returns() {
            let input =
                InputSpan::new_extra("~~~\r\nLine with CR\r\n~~~\r\nrest", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse multi-line note with carriage returns");
            assert_eq!(kind, NoteKind::MultiLine);
            assert!(matched.lexeme_str.contains("Line with CR"));
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
            assert!(matched.lexeme_str.contains("Line with 世界 characters"));
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn with_emoji() {
            let input =
                InputSpan::new_extra("~~~\nLine with 😀 emoji\n~~~\nrest", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse multi-line note with emoji");
            assert_eq!(kind, NoteKind::MultiLine);
            assert!(matched.lexeme_str.contains("Line with 😀 emoji"));
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
            assert!(matched.lexeme_str.contains("Line with ~ tilde in content"));
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
            assert!(
                matched
                    .lexeme_str
                    .contains("Line with ~~ partial delimiter")
            );
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
            assert_eq!(matched.lexeme_str, "~~");
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
            assert!(matched.lexeme_str.contains("Line 1"));
            assert_eq!(rest.fragment(), &"Line 2\n~~~\nrest");
        }

        #[test]
        fn mismatched_delimiters() {
            let input = InputSpan::new_extra("~~~~\ncontent\n~~~\nrest", Config::default());
            let (rest, (matched, kind)) =
                note(input).expect("should parse multi-line note with mismatched delimiters");
            assert_eq!(kind, NoteKind::MultiLine);
            assert!(matched.lexeme_str.contains("content"));
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

        #[test]
        fn with_characters_after_closing_delimiter() {
            let input = InputSpan::new_extra("~~~\ncontent\n~~~foo", Config::default());
            let res = note(input);
            let Err(nom::Err::Failure(token_error)) = res else {
                panic!("expected TokenError::Incomplete(InvalidClosingDelimiter), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Incomplete(IncompleteKind::InvalidClosingDelimiter)
            ));
        }
    }
}
