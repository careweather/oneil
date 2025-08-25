//! Provides parsers for structural elements in the Oneil language.
//!
//! This module currently contains parsers for end-of-line tokens, which include:
//! - Line breaks (`\n`)
//! - Comments (starting with `#` and continuing to end of line)
//! - End-of-file markers
//! - Any combination of the above with optional whitespace

use nom::{
    Parser as _,
    character::complete::{char, line_ending, not_line_ending},
    combinator::{eof, opt, recognize},
    multi::many0,
};

use crate::token::{
    Result, Span,
    error::{ErrorHandlingParser, TokenError},
    util::{Token, inline_whitespace},
};

/// Parses a line break character or sequence.
///
/// This function recognizes various line ending sequences including `\n`, `\r\n`,
/// and `\r`. It's used internally by the `end_of_line` parser.
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns the span containing the line break sequence.
fn linebreak(input: Span<'_>) -> Result<'_, Span<'_>, TokenError> {
    line_ending.parse(input)
}

/// Parses the end of the input file.
///
/// This function succeeds only when there is no more input to parse.
/// It's used internally by the `end_of_line` parser to handle files
/// that end without a final newline.
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns an empty span when at end of file.
fn end_of_file(input: Span<'_>) -> Result<'_, Span<'_>, TokenError> {
    eof.parse(input)
}

/// Parses a comment line starting with `#`.
///
/// This function recognizes comments that start with `#` and continue
/// until the end of the line or end of file. Comments are treated
/// as structural elements and can be part of end-of-line sequences.
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns the span containing the comment including the `#` and newline.
fn comment(input: Span<'_>) -> Result<'_, Span<'_>, TokenError> {
    recognize((char('#'), not_line_ending, line_ending.or(eof))).parse(input)
}

/// Parses one or more linebreaks, comments, or end-of-file markers, including trailing whitespace.
///
/// This function recognizes any combination of:
/// - Line breaks (`\n`, `\r\n`, `\r`)
/// - Comments (starting with `#` and continuing to end of line)
/// - End-of-file markers
///
/// It also captures any whitespace that follows these elements. This is used to properly
/// handle line endings and comments in the Oneil language syntax.
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns a token containing the first line break/comment/EOF marker and any trailing
/// whitespace including subsequent line breaks or comments.
pub fn end_of_line(input: Span<'_>) -> Result<'_, Token<'_>, TokenError> {
    let (rest, first_line_break) = linebreak
        .or(comment)
        .or(end_of_file)
        .convert_error_to(TokenError::expected_end_of_line)
        .parse(input)?;

    let (rest, rest_whitespace) = recognize((
        inline_whitespace,
        many0((linebreak.or(comment), inline_whitespace)),
        opt(end_of_file),
    ))
    .parse(rest)?;

    Ok((rest, Token::new(first_line_break, rest_whitespace)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;
    use crate::token::error::{ExpectKind, TokenErrorKind};

    mod linebreak_tests {
        use super::*;

        #[test]
        fn test_unix_newline() {
            let input = Span::new_extra("\nrest", Config::default());
            let (rest, matched) = linebreak(input).expect("should parse unix newline");
            assert_eq!(matched.fragment(), &"\n");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_windows_newline() {
            let input = Span::new_extra("\r\nrest", Config::default());
            let (rest, matched) = linebreak(input).expect("should parse windows newline");
            assert_eq!(matched.fragment(), &"\r\n");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_no_newline() {
            let input = Span::new_extra("no newline", Config::default());
            let res = linebreak(input);
            match res {
                Err(nom::Err::Error(token_error)) => {
                    assert!(matches!(token_error.kind, TokenErrorKind::NomError(_)));
                }
                _ => panic!("expected TokenError::NomError(_), got {res:?}"),
            }
        }

        #[test]
        fn test_empty_input() {
            let input = Span::new_extra("", Config::default());
            let res = linebreak(input);
            match res {
                Err(nom::Err::Error(token_error)) => {
                    assert!(matches!(token_error.kind, TokenErrorKind::NomError(_)));
                }
                _ => panic!("expected TokenError::NomError(_), got {res:?}"),
            }
        }

        #[test]
        fn test_whitespace_before_newline() {
            let input = Span::new_extra("   \nrest", Config::default());
            let res = linebreak(input);
            match res {
                Err(nom::Err::Error(token_error)) => {
                    assert!(matches!(token_error.kind, TokenErrorKind::NomError(_)));
                }
                _ => panic!("expected TokenError::NomError(_), got {res:?}"),
            }
        }
    }

    mod comment_tests {
        use super::*;

        #[test]
        fn test_basic_comment() {
            let input = Span::new_extra("# this is a comment\nrest", Config::default());
            let (rest, matched) = comment(input).expect("should parse basic comment");
            assert_eq!(matched.fragment(), &"# this is a comment\n");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_comment_at_eof() {
            let input = Span::new_extra("# only comment", Config::default());
            let (rest, matched) = comment(input).expect("should parse comment at EOF");
            assert_eq!(matched.fragment(), &"# only comment");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_empty_comment() {
            let input = Span::new_extra("#\nrest", Config::default());
            let (rest, matched) = comment(input).expect("should parse empty comment");
            assert_eq!(matched.fragment(), &"#\n");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_comment_with_special_characters() {
            let input = Span::new_extra("# comment with @#$% symbols\nrest", Config::default());
            let (rest, matched) =
                comment(input).expect("should parse comment with special characters");
            assert_eq!(matched.fragment(), &"# comment with @#$% symbols\n");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_comment_with_numbers() {
            let input = Span::new_extra("# comment with 123 numbers\nrest", Config::default());
            let (rest, matched) = comment(input).expect("should parse comment with numbers");
            assert_eq!(matched.fragment(), &"# comment with 123 numbers\n");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_comment_with_unicode() {
            let input = Span::new_extra("# comment with 世界 characters\nrest", Config::default());
            let (rest, matched) = comment(input).expect("should parse comment with unicode");
            assert_eq!(matched.fragment(), &"# comment with 世界 characters\n");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_comment_with_emoji() {
            let input = Span::new_extra("# comment with 😀 emoji\nrest", Config::default());
            let (rest, matched) = comment(input).expect("should parse comment with emoji");
            assert_eq!(matched.fragment(), &"# comment with 😀 emoji\n");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_comment_with_tabs() {
            let input = Span::new_extra("# comment\twith\ttabs\nrest", Config::default());
            let (rest, matched) = comment(input).expect("should parse comment with tabs");
            assert_eq!(matched.fragment(), &"# comment\twith\ttabs\n");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_comment_with_carriage_return() {
            let input = Span::new_extra("# comment\r\nrest", Config::default());
            let (rest, matched) =
                comment(input).expect("should parse comment with carriage return");
            assert_eq!(matched.fragment(), &"# comment\r\n");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_no_hash() {
            let input = Span::new_extra("not a comment\n", Config::default());
            let res = comment(input);
            match res {
                Err(nom::Err::Error(token_error)) => {
                    assert!(matches!(token_error.kind, TokenErrorKind::NomError(_)));
                }
                _ => panic!("expected TokenError::NomError(_), got {res:?}"),
            }
        }

        #[test]
        fn test_hash_without_content() {
            let input = Span::new_extra("#", Config::default());
            let (rest, matched) = comment(input).expect("should parse hash without content");
            assert_eq!(matched.fragment(), &"#");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_empty_input() {
            let input = Span::new_extra("", Config::default());
            let res = comment(input);
            match res {
                Err(nom::Err::Error(token_error)) => {
                    assert!(matches!(token_error.kind, TokenErrorKind::NomError(_)));
                }
                _ => panic!("expected TokenError::NomError(_), got {res:?}"),
            }
        }

        #[test]
        fn test_whitespace_before_hash() {
            let input = Span::new_extra("   # comment\nrest", Config::default());
            let res = comment(input);
            match res {
                Err(nom::Err::Error(token_error)) => {
                    assert!(matches!(token_error.kind, TokenErrorKind::NomError(_)));
                }
                _ => panic!("expected TokenError::NomError(_), got {res:?}"),
            }
        }
    }

    mod end_of_file_tests {
        use super::*;

        #[test]
        fn test_empty_input() {
            let input = Span::new_extra("", Config::default());
            let (rest, matched) = end_of_file(input).expect("should parse end of file");
            assert_eq!(matched.fragment(), &"");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_not_empty() {
            let input = Span::new_extra("not empty", Config::default());
            let res = end_of_file(input);
            match res {
                Err(nom::Err::Error(token_error)) => {
                    assert!(matches!(token_error.kind, TokenErrorKind::NomError(_)));
                }
                _ => panic!("expected TokenError::NomError(_), got {res:?}"),
            }
        }

        #[test]
        fn test_whitespace_only() {
            let input = Span::new_extra("   ", Config::default());
            let res = end_of_file(input);
            match res {
                Err(nom::Err::Error(token_error)) => {
                    assert!(matches!(token_error.kind, TokenErrorKind::NomError(_)));
                }
                _ => panic!("expected TokenError::NomError(_), got {res:?}"),
            }
        }

        #[test]
        fn test_newline() {
            let input = Span::new_extra("\n", Config::default());
            let res = end_of_file(input);
            match res {
                Err(nom::Err::Error(token_error)) => {
                    assert!(matches!(token_error.kind, TokenErrorKind::NomError(_)));
                }
                _ => panic!("expected TokenError::NomError(_), got {res:?}"),
            }
        }

        #[test]
        fn test_comment() {
            let input = Span::new_extra("# comment", Config::default());
            let res = end_of_file(input);
            match res {
                Err(nom::Err::Error(token_error)) => {
                    assert!(matches!(token_error.kind, TokenErrorKind::NomError(_)));
                }
                _ => panic!("expected TokenError::NomError(_), got {res:?}"),
            }
        }
    }

    mod end_of_line_tests {
        use super::*;

        #[test]
        fn test_single_linebreak() {
            let input = Span::new_extra("\nrest", Config::default());
            let (rest, matched) = end_of_line(input).expect("should parse single linebreak");
            assert_eq!(rest.fragment(), &"rest");
            assert_eq!(matched.lexeme(), "\n");
            assert!(matched.whitespace().is_empty());
        }

        #[test]
        fn test_single_comment() {
            let input = Span::new_extra("# comment\nrest", Config::default());
            let (rest, matched) = end_of_line(input).expect("should parse single comment");
            assert_eq!(rest.fragment(), &"rest");
            assert_eq!(matched.lexeme(), "# comment\n");
            assert!(matched.whitespace().is_empty());
        }

        #[test]
        fn test_eof() {
            let input = Span::new_extra("", Config::default());
            let (rest, matched) = end_of_line(input).expect("should parse EOF");
            assert_eq!(rest.fragment(), &"");
            assert_eq!(matched.lexeme(), "");
            assert_eq!(matched.whitespace(), "");
        }

        #[test]
        fn test_multiple_linebreaks() {
            let input = Span::new_extra("\n\n\nrest", Config::default());
            let (rest, matched) = end_of_line(input).expect("should parse multiple linebreaks");
            assert_eq!(rest.fragment(), &"rest");
            assert_eq!(matched.lexeme(), "\n");
            assert!(matched.whitespace().contains("\n\n"));
        }

        #[test]
        fn test_multiple_comments() {
            let input = Span::new_extra("# foo\n# bar\nrest", Config::default());
            let (rest, matched) = end_of_line(input).expect("should parse multiple comments");
            assert_eq!(rest.fragment(), &"rest");
            assert_eq!(matched.lexeme(), "# foo\n");
            assert!(matched.whitespace().contains("# bar\n"));
        }

        #[test]
        fn test_mixed_linebreaks_and_comments() {
            let input = Span::new_extra("\n# foo\n\n# bar\nrest", Config::default());
            let (rest, matched) =
                end_of_line(input).expect("should parse mixed linebreaks and comments");
            assert_eq!(rest.fragment(), &"rest");
            assert_eq!(matched.lexeme(), "\n");
            assert!(matched.whitespace().contains("# foo\n"));
            assert!(matched.whitespace().contains("# bar\n"));
        }

        #[test]
        fn test_with_whitespace_between() {
            let input = Span::new_extra("\n   # foo   \n   \n   # bar   \nrest", Config::default());
            let (rest, matched) = end_of_line(input).expect("should parse with whitespace between");
            assert_eq!(rest.fragment(), &"rest");
            assert_eq!(matched.lexeme(), "\n");
            assert!(matched.whitespace().contains("# foo"));
            assert!(matched.whitespace().contains("# bar"));
        }

        #[test]
        fn test_with_tabs_between() {
            let input = Span::new_extra("\n\t# foo\t\n\t\n\t# bar\t\nrest", Config::default());
            let (rest, matched) = end_of_line(input).expect("should parse with tabs between");
            assert_eq!(rest.fragment(), &"rest");
            assert_eq!(matched.lexeme(), "\n");
            assert!(matched.whitespace().contains("# foo"));
            assert!(matched.whitespace().contains("# bar"));
        }

        #[test]
        fn test_ending_with_eof() {
            let input = Span::new_extra("\n# comment\n\n", Config::default());
            let (rest, matched) = end_of_line(input).expect("should parse ending with EOF");
            assert_eq!(rest.fragment(), &"");
            assert_eq!(matched.lexeme(), "\n");
            assert!(matched.whitespace().contains("# comment\n"));
        }

        #[test]
        fn test_windows_line_endings() {
            let input = Span::new_extra("\r\n# comment\r\nrest", Config::default());
            let (rest, matched) = end_of_line(input).expect("should parse windows line endings");
            assert_eq!(rest.fragment(), &"rest");
            assert_eq!(matched.lexeme(), "\r\n");
            assert!(matched.whitespace().contains("# comment\r\n"));
        }

        #[test]
        fn test_mixed_line_endings() {
            let input = Span::new_extra(
                "\n# unix comment\r\n# windows comment\nrest",
                Config::default(),
            );
            let (rest, matched) = end_of_line(input).expect("should parse mixed line endings");
            assert_eq!(rest.fragment(), &"rest");
            assert_eq!(matched.lexeme(), "\n");
            assert!(matched.whitespace().contains("# unix comment"));
            assert!(matched.whitespace().contains("# windows comment"));
        }

        #[test]
        fn test_comments_with_special_characters() {
            let input = Span::new_extra(
                "# comment with @#$% symbols\n# another comment with 123\nrest",
                Config::default(),
            );
            let (rest, matched) =
                end_of_line(input).expect("should parse comments with special characters");
            assert_eq!(rest.fragment(), &"rest");
            assert_eq!(matched.lexeme(), "# comment with @#$% symbols\n");
            assert!(
                matched
                    .whitespace()
                    .contains("# another comment with 123\n")
            );
        }

        #[test]
        fn test_comments_with_unicode() {
            let input = Span::new_extra(
                "# comment with 世界 characters\n# another comment with 😀 emoji\nrest",
                Config::default(),
            );
            let (rest, matched) = end_of_line(input).expect("should parse comments with unicode");
            assert_eq!(rest.fragment(), &"rest");
            assert_eq!(matched.lexeme(), "# comment with 世界 characters\n");
            assert!(
                matched
                    .whitespace()
                    .contains("# another comment with 😀 emoji\n")
            );
        }

        #[test]
        fn test_no_end_of_line() {
            let input = Span::new_extra("no end of line", Config::default());
            let res = end_of_line(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::EndOfLine)
                )),
                _ => panic!("expected TokenError::Expect(EndOfLine), got {res:?}"),
            }
        }

        #[test]
        fn test_whitespace_only() {
            let input = Span::new_extra("   ", Config::default());
            let res = end_of_line(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::EndOfLine)
                )),
                _ => panic!("expected TokenError::Expect(EndOfLine), got {res:?}"),
            }
        }

        #[test]
        fn test_whitespace_before_linebreak() {
            let input = Span::new_extra("   \nrest", Config::default());
            let res = end_of_line(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::EndOfLine)
                )),
                _ => panic!("expected TokenError::Expect(EndOfLine), got {res:?}"),
            }
        }

        #[test]
        fn test_whitespace_before_comment() {
            let input = Span::new_extra("   # comment\nrest", Config::default());
            let res = end_of_line(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::EndOfLine)
                )),
                _ => panic!("expected TokenError::Expect(EndOfLine), got {res:?}"),
            }
        }
    }
}
