//! Parser for notes in an Oneil program

use nom::Parser;
use nom::combinator::all_consuming;
use oneil_ast::{Node, Note, NoteNode};

use crate::error::{ErrorHandlingParser, ParserError};
use crate::token::note::{NoteKind, note as note_token};
use crate::util::{InputSpan, Result};

/// Parse a note, which can be either a single-line note starting with `~`
/// or a multi-line note delimited by `~~~`.
///
/// This function **may not consume the complete input**.
pub fn parse(input: InputSpan<'_>) -> Result<'_, NoteNode, ParserError> {
    note(input)
}

/// Parse a note, which can be either a single-line note starting with `~`
/// or a multi-line note delimited by `~~~`.
///
/// This function **fails if the complete input is not consumed**.
pub fn parse_complete(input: InputSpan<'_>) -> Result<'_, NoteNode, ParserError> {
    all_consuming(note).parse(input)
}

/// Parses a note token and converts it to a note node.
///
/// This function parses either a single-line note (starting with `~`) or
/// a multi-line note (delimited by `~~~`) and converts the token into
/// a proper note node with the content extracted and trimmed.
///
/// For single-line notes, the leading `~` is removed and the content is trimmed.
/// For multi-line notes, the leading and trailing `~~~` are removed and the
/// content is trimmed.
fn note(input: InputSpan<'_>) -> Result<'_, NoteNode, ParserError> {
    let (rest, (token, kind)) = note_token
        .convert_error_to(ParserError::expect_note)
        .parse(input)?;

    let note = match kind {
        NoteKind::SingleLine => {
            let content = token.lexeme().trim_start_matches('~').trim();
            Node::new(&token, Note::new(content.to_string()))
        }
        NoteKind::MultiLine => {
            let content = token.lexeme().trim_matches('~').trim();
            Node::new(&token, Note::new(content.to_string()))
        }
    };

    Ok((rest, note))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;
    use crate::error::reason::{ExpectKind, ParserErrorReason};
    use crate::token::error::{IncompleteKind, TokenErrorKind};
    use oneil_ast::AstSpan;

    #[test]
    fn single_line_note() {
        let input = InputSpan::new_extra("~ This is a note\nrest", Config::default());
        let (rest, note) = parse(input).expect("should parse single line note");
        assert_eq!(note.node_span(), AstSpan::new(0, 16, 1));
        assert_eq!(note.node_value(), &Note::new("This is a note".to_string()));
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn single_line_note_at_eof() {
        let input = InputSpan::new_extra("~ note", Config::default());
        let (rest, note) = parse(input).expect("should parse single line note at EOF");
        assert_eq!(note.node_span(), AstSpan::new(0, 6, 0));
        assert_eq!(note.node_value(), &Note::new("note".to_string()));
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn multi_line_note() {
        let input = InputSpan::new_extra("~~~\nLine 1\nLine 2\n~~~\nrest", Config::default());
        let (rest, note) = parse(input).expect("should parse multi-line note");
        assert_eq!(note.node_span(), AstSpan::new(0, 21, 1));
        assert_eq!(note.node_value(), &Note::new("Line 1\nLine 2".to_string()));
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn multi_line_note_extra_tildes() {
        let input = InputSpan::new_extra("~~~~~\nfoo\nbar\n~~~~~\nrest", Config::default());
        let (rest, note) = parse(input).expect("should parse multi-line note with extra tildes");
        assert_eq!(note.node_span(), AstSpan::new(0, 19, 1));
        assert_eq!(note.node_value(), &Note::new("foo\nbar".to_string()));
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn multi_line_note_empty() {
        let input = InputSpan::new_extra("~~~\n~~~\nrest", Config::default());
        let (rest, note) = parse(input).expect("should parse empty multi-line note");
        assert_eq!(note.node_span(), AstSpan::new(0, 7, 1));
        assert_eq!(note.node_value(), &Note::new(String::new()));
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn multi_line_note_unclosed() {
        let input = InputSpan::new_extra("~~~\nUnclosed note\n", Config::default());
        let result = parse(input);
        assert!(result.is_err(), "should not parse unclosed multi-line note");
    }

    #[test]
    fn invalid_note() {
        let input = InputSpan::new_extra("not a note", Config::default());
        let result = parse(input);
        assert!(result.is_err(), "should not parse invalid note");
    }

    #[test]
    fn parse_complete_single_line_success() {
        let input = InputSpan::new_extra("~ This is a note\n", Config::default());
        let (rest, note) = parse_complete(input).expect("should parse single line note");
        assert_eq!(note.node_span(), AstSpan::new(0, 16, 1));
        assert_eq!(note.node_value(), &Note::new("This is a note".to_string()));
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn parse_complete_multi_line_success() {
        let input = InputSpan::new_extra("~~~\nLine 1\nLine 2\n~~~\n", Config::default());
        let (rest, note) = parse_complete(input).expect("should parse multi-line note");
        assert_eq!(note.node_span(), AstSpan::new(0, 21, 1));
        assert_eq!(note.node_value(), &Note::new("Line 1\nLine 2".to_string()));
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn parse_complete_with_remaining_input() {
        let input = InputSpan::new_extra("~ This is a note\nrest", Config::default());
        let result = parse_complete(input);
        assert!(
            result.is_err(),
            "should not parse complete with remaining input for single line note"
        );
    }

    mod single_line_error {
        use super::*;

        #[test]
        fn single_line_note_missing_tilde() {
            let input = InputSpan::new_extra("This is not a note\n", Config::default());
            let result = parse(input);
            let Err(nom::Err::Error(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 0);
            assert!(matches!(
                error.reason,
                ParserErrorReason::Expect(ExpectKind::Note)
            ));
        }

        #[test]
        fn single_line_note_empty_input() {
            let input = InputSpan::new_extra("", Config::default());
            let result = parse(input);

            let Err(nom::Err::Error(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 0);
            assert!(matches!(
                error.reason,
                ParserErrorReason::Expect(ExpectKind::Note)
            ));
        }

        #[test]
        fn single_line_note_whitespace_only() {
            let input = InputSpan::new_extra("   \n", Config::default());
            let result = parse(input);

            let Err(nom::Err::Error(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 3); // fails after the whitespace
            assert!(matches!(
                error.reason,
                ParserErrorReason::Expect(ExpectKind::Note)
            ));
        }
    }

    mod multi_line_error {
        use super::*;

        #[test]
        fn multi_line_note_missing_opening_delimiter() {
            let input = InputSpan::new_extra("content\n~~~\n", Config::default());
            let result = parse(input);

            let Err(nom::Err::Error(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 0);
            assert!(matches!(
                error.reason,
                ParserErrorReason::Expect(ExpectKind::Note)
            ));
        }

        #[test]
        fn multi_line_note_missing_closing_delimiter() {
            let input = InputSpan::new_extra("~~~\ncontent\n", Config::default());
            let result = parse(input);
            let Err(nom::Err::Failure(error)) = result else {
                panic!("Expected failure for unclosed multi-line note");
            };

            assert_eq!(error.error_offset, 12); // error at end of content
            let ParserErrorReason::TokenError(TokenErrorKind::Incomplete(
                IncompleteKind::UnclosedNote {
                    delimiter_start_offset,
                    delimiter_length,
                },
            )) = error.reason
            else {
                panic!("Unexpected reason {:?}", error.reason);
            };

            assert_eq!(delimiter_start_offset, 0);
            assert_eq!(delimiter_length, 3);
        }

        #[test]
        fn multi_line_note_empty_input() {
            let input = InputSpan::new_extra("", Config::default());
            let result = parse(input);

            let Err(nom::Err::Error(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 0);
            assert!(matches!(
                error.reason,
                ParserErrorReason::Expect(ExpectKind::Note)
            ));
        }

        #[test]
        fn multi_line_note_whitespace_only() {
            let input = InputSpan::new_extra("   \n", Config::default());
            let result = parse(input);

            let Err(nom::Err::Error(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 3); // fails after the whitespace
            assert!(matches!(
                error.reason,
                ParserErrorReason::Expect(ExpectKind::Note)
            ));
        }
    }

    mod parse_complete_error {
        use super::*;

        #[test]
        fn parse_complete_invalid_input() {
            let input = InputSpan::new_extra("invalid input", Config::default());
            let result = parse_complete(input);

            let Err(nom::Err::Error(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 0);
            assert!(matches!(
                error.reason,
                ParserErrorReason::Expect(ExpectKind::Note)
            ));
        }
    }
}
