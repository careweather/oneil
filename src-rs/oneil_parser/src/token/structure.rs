//! Provides parsers for structural elements in the Oneil language.

use nom::{
    Parser as _,
    character::complete::{char, line_ending, not_line_ending},
    combinator::{eof, opt, recognize},
    multi::many0,
};
use oneil_shared::span::{SourceLocation, Span};

use crate::token::{
    InputSpan, Result,
    error::{ErrorHandlingParser, TokenError},
    util::{Token, inline_whitespace},
};

/// Parses a line break character or sequence.
///
/// This function recognizes various line ending sequences including `\n`, `\r\n`,
/// and `\r`. It's used internally by the `end_of_line` parser.
fn linebreak(input: InputSpan<'_>) -> Result<'_, InputSpan<'_>, TokenError> {
    line_ending.parse(input)
}

/// Parses the end of the input file.
///
/// This function succeeds only when there is no more input to parse.
/// It's used internally by the `end_of_line` parser to handle files
/// that end without a final newline.
fn end_of_file(input: InputSpan<'_>) -> Result<'_, InputSpan<'_>, TokenError> {
    eof.parse(input)
}

/// Parses a comment line starting with `#`.
fn comment(input: InputSpan<'_>) -> Result<'_, InputSpan<'_>, TokenError> {
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
pub fn end_of_line(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    // because the end of line is whitespace, the lexeme is empty
    let lexeme_str = &input.fragment()[..0];

    let lexeme_start_line = usize::try_from(input.location_line())
        .expect("usize should be greater than or equal to u32");
    let lexeme_start = SourceLocation {
        offset: input.location_offset(),
        line: lexeme_start_line,
        column: input.get_column(),
    };

    let lexeme_end_line = usize::try_from(input.location_line())
        .expect("usize should be greater than or equal to u32");
    let lexeme_end = SourceLocation {
        offset: input.location_offset(),
        line: lexeme_end_line,
        column: input.get_column(),
    };

    let lexeme_span = Span::new(lexeme_start, lexeme_end);

    // parse any whitespace, including the linebreak
    let (rest, whitespace) = recognize(|input| {
        // parse the first linebreak, comment, or end of file
        let (rest, _) = linebreak
            .or(comment)
            .or(end_of_file)
            .convert_error_to(TokenError::expected_end_of_line)
            .parse(input)?;

        // parse any additional linebreaks or comments
        let (rest, _) = inline_whitespace.parse(rest)?;

        let (rest, _) = many0(|input| {
            let (rest, _) = linebreak.or(comment).parse(input)?;
            let (rest, _) = inline_whitespace.parse(rest)?;
            Ok((rest, ()))
        })
        .parse(rest)?;

        // parse the end of file, if present
        let (rest, _) = opt(end_of_file).parse(rest)?;

        Ok((rest, ()))
    })
    .parse(input)?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;
    use crate::token::error::{ExpectKind, TokenErrorKind};

    mod linebreak {
        use super::*;

        #[test]
        fn unix_newline() {
            let input = InputSpan::new_extra("\nrest", Config::default());
            let (rest, matched) = linebreak(input).expect("should parse unix newline");
            assert_eq!(matched.fragment(), &"\n");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn windows_newline() {
            let input = InputSpan::new_extra("\r\nrest", Config::default());
            let (rest, matched) = linebreak(input).expect("should parse windows newline");
            assert_eq!(matched.fragment(), &"\r\n");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn no_newline() {
            let input = InputSpan::new_extra("no newline", Config::default());
            let res = linebreak(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::NomError(_), got {res:?}");
            };

            assert!(matches!(token_error.kind, TokenErrorKind::NomError(_)));
        }

        #[test]
        fn empty_input() {
            let input = InputSpan::new_extra("", Config::default());
            let res = linebreak(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::NomError(_), got {res:?}");
            };

            assert!(matches!(token_error.kind, TokenErrorKind::NomError(_)));
        }

        #[test]
        fn whitespace_before_newline() {
            let input = InputSpan::new_extra("   \nrest", Config::default());
            let res = linebreak(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::NomError(_), got {res:?}");
            };

            assert!(matches!(token_error.kind, TokenErrorKind::NomError(_)));
        }
    }

    mod comment {
        use super::*;

        #[test]
        fn basic_comment() {
            let input = InputSpan::new_extra("# this is a comment\nrest", Config::default());
            let (rest, matched) = comment(input).expect("should parse basic comment");
            assert_eq!(matched.fragment(), &"# this is a comment\n");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn comment_at_eof() {
            let input = InputSpan::new_extra("# only comment", Config::default());
            let (rest, matched) = comment(input).expect("should parse comment at EOF");
            assert_eq!(matched.fragment(), &"# only comment");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn empty_comment() {
            let input = InputSpan::new_extra("#\nrest", Config::default());
            let (rest, matched) = comment(input).expect("should parse empty comment");
            assert_eq!(matched.fragment(), &"#\n");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn comment_with_special_characters() {
            let input =
                InputSpan::new_extra("# comment with @#$% symbols\nrest", Config::default());
            let (rest, matched) =
                comment(input).expect("should parse comment with special characters");
            assert_eq!(matched.fragment(), &"# comment with @#$% symbols\n");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn comment_with_numbers() {
            let input = InputSpan::new_extra("# comment with 123 numbers\nrest", Config::default());
            let (rest, matched) = comment(input).expect("should parse comment with numbers");
            assert_eq!(matched.fragment(), &"# comment with 123 numbers\n");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn comment_with_unicode() {
            let input =
                InputSpan::new_extra("# comment with 世界 characters\nrest", Config::default());
            let (rest, matched) = comment(input).expect("should parse comment with unicode");
            assert_eq!(matched.fragment(), &"# comment with 世界 characters\n");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn comment_with_emoji() {
            let input = InputSpan::new_extra("# comment with 😀 emoji\nrest", Config::default());
            let (rest, matched) = comment(input).expect("should parse comment with emoji");
            assert_eq!(matched.fragment(), &"# comment with 😀 emoji\n");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn comment_with_tabs() {
            let input = InputSpan::new_extra("# comment\twith\ttabs\nrest", Config::default());
            let (rest, matched) = comment(input).expect("should parse comment with tabs");
            assert_eq!(matched.fragment(), &"# comment\twith\ttabs\n");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn comment_with_carriage_return() {
            let input = InputSpan::new_extra("# comment\r\nrest", Config::default());
            let (rest, matched) =
                comment(input).expect("should parse comment with carriage return");
            assert_eq!(matched.fragment(), &"# comment\r\n");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn no_hash() {
            let input = InputSpan::new_extra("not a comment\n", Config::default());
            let res = comment(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::NomError(_), got {res:?}");
            };

            assert!(matches!(token_error.kind, TokenErrorKind::NomError(_)));
        }

        #[test]
        fn hash_without_content() {
            let input = InputSpan::new_extra("#", Config::default());
            let (rest, matched) = comment(input).expect("should parse hash without content");
            assert_eq!(matched.fragment(), &"#");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn empty_input() {
            let input = InputSpan::new_extra("", Config::default());
            let res = comment(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::NomError(_), got {res:?}");
            };

            assert!(matches!(token_error.kind, TokenErrorKind::NomError(_)));
        }

        #[test]
        fn whitespace_before_hash() {
            let input = InputSpan::new_extra("   # comment\nrest", Config::default());
            let res = comment(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::NomError(_), got {res:?}");
            };

            assert!(matches!(token_error.kind, TokenErrorKind::NomError(_)));
        }
    }

    mod end_of_file {
        use super::*;

        #[test]
        fn empty_input() {
            let input = InputSpan::new_extra("", Config::default());
            let (rest, matched) = end_of_file(input).expect("should parse end of file");
            assert_eq!(matched.fragment(), &"");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn not_empty() {
            let input = InputSpan::new_extra("not empty", Config::default());
            let res = end_of_file(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::NomError(_), got {res:?}");
            };

            assert!(matches!(token_error.kind, TokenErrorKind::NomError(_)));
        }

        #[test]
        fn whitespace_only() {
            let input = InputSpan::new_extra("   ", Config::default());
            let res = end_of_file(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::NomError(_), got {res:?}");
            };

            assert!(matches!(token_error.kind, TokenErrorKind::NomError(_)));
        }

        #[test]
        fn newline() {
            let input = InputSpan::new_extra("\n", Config::default());
            let res = end_of_file(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::NomError(_), got {res:?}");
            };

            assert!(matches!(token_error.kind, TokenErrorKind::NomError(_)));
        }

        #[test]
        fn comment() {
            let input = InputSpan::new_extra("# comment", Config::default());
            let res = end_of_file(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::NomError(_), got {res:?}");
            };

            assert!(matches!(token_error.kind, TokenErrorKind::NomError(_)));
        }
    }

    mod end_of_line {
        use super::*;

        #[test]
        fn single_linebreak() {
            let input = InputSpan::new_extra("\nrest", Config::default());
            let (rest, matched) = end_of_line(input).expect("should parse single linebreak");
            assert_eq!(rest.fragment(), &"rest");
            assert_eq!(matched.lexeme_str, "");
        }

        #[test]
        fn single_comment() {
            let input = InputSpan::new_extra("# comment\nrest", Config::default());
            let (rest, matched) = end_of_line(input).expect("should parse single comment");
            assert_eq!(rest.fragment(), &"rest");
            assert_eq!(matched.lexeme_str, "");
        }

        #[test]
        fn eof() {
            let input = InputSpan::new_extra("", Config::default());
            let (rest, matched) = end_of_line(input).expect("should parse EOF");
            assert_eq!(rest.fragment(), &"");
            assert_eq!(matched.lexeme_str, "");
        }

        #[test]
        fn multiple_linebreaks() {
            let input = InputSpan::new_extra("\n\n\nrest", Config::default());
            let (rest, matched) = end_of_line(input).expect("should parse multiple linebreaks");
            assert_eq!(rest.fragment(), &"rest");
            assert_eq!(matched.lexeme_str, "");
        }

        #[test]
        fn multiple_comments() {
            let input = InputSpan::new_extra("# foo\n# bar\nrest", Config::default());
            let (rest, matched) = end_of_line(input).expect("should parse multiple comments");
            assert_eq!(rest.fragment(), &"rest");
            assert_eq!(matched.lexeme_str, "");
        }

        #[test]
        fn mixed_linebreaks_and_comments() {
            let input = InputSpan::new_extra("\n# foo\n\n# bar\nrest", Config::default());
            let (rest, matched) =
                end_of_line(input).expect("should parse mixed linebreaks and comments");
            assert_eq!(rest.fragment(), &"rest");
            assert_eq!(matched.lexeme_str, "");
        }

        #[test]
        fn with_whitespace_between() {
            let input =
                InputSpan::new_extra("\n   # foo   \n   \n   # bar   \nrest", Config::default());
            let (rest, matched) = end_of_line(input).expect("should parse with whitespace between");
            assert_eq!(rest.fragment(), &"rest");
            assert_eq!(matched.lexeme_str, "");
        }

        #[test]
        fn with_tabs_between() {
            let input = InputSpan::new_extra("\n\t# foo\t\n\t\n\t# bar\t\nrest", Config::default());
            let (rest, matched) = end_of_line(input).expect("should parse with tabs between");
            assert_eq!(rest.fragment(), &"rest");
            assert_eq!(matched.lexeme_str, "");
        }

        #[test]
        fn ending_with_eof() {
            let input = InputSpan::new_extra("\n# comment\n\n", Config::default());
            let (rest, matched) = end_of_line(input).expect("should parse ending with EOF");
            assert_eq!(rest.fragment(), &"");
            assert_eq!(matched.lexeme_str, "");
        }

        #[test]
        fn windows_line_endings() {
            let input = InputSpan::new_extra("\r\n# comment\r\nrest", Config::default());
            let (rest, matched) = end_of_line(input).expect("should parse windows line endings");
            assert_eq!(rest.fragment(), &"rest");
            assert_eq!(matched.lexeme_str, "");
        }

        #[test]
        fn mixed_line_endings() {
            let input = InputSpan::new_extra(
                "\n# unix comment\r\n# windows comment\nrest",
                Config::default(),
            );
            let (rest, matched) = end_of_line(input).expect("should parse mixed line endings");
            assert_eq!(rest.fragment(), &"rest");
            assert_eq!(matched.lexeme_str, "");
        }

        #[test]
        fn comments_with_special_characters() {
            let input = InputSpan::new_extra(
                "# comment with @#$% symbols\n# another comment with 123\nrest",
                Config::default(),
            );
            let (rest, matched) =
                end_of_line(input).expect("should parse comments with special characters");
            assert_eq!(rest.fragment(), &"rest");
            assert_eq!(matched.lexeme_str, "");
        }

        #[test]
        fn comments_with_unicode() {
            let input = InputSpan::new_extra(
                "# comment with 世界 characters\n# another comment with 😀 emoji\nrest",
                Config::default(),
            );
            let (rest, matched) = end_of_line(input).expect("should parse comments with unicode");
            assert_eq!(rest.fragment(), &"rest");
            assert_eq!(matched.lexeme_str, "");
        }

        #[test]
        fn no_end_of_line() {
            let input = InputSpan::new_extra("no end of line", Config::default());
            let res = end_of_line(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(EndOfLine), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::EndOfLine)
            ));
        }

        #[test]
        fn whitespace_only() {
            let input = InputSpan::new_extra("   ", Config::default());
            let res = end_of_line(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(EndOfLine), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::EndOfLine)
            ));
        }

        #[test]
        fn whitespace_before_linebreak() {
            let input = InputSpan::new_extra("   \nrest", Config::default());
            let res = end_of_line(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(EndOfLine), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::EndOfLine)
            ));
        }

        #[test]
        fn whitespace_before_comment() {
            let input = InputSpan::new_extra("   # comment\nrest", Config::default());
            let res = end_of_line(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(EndOfLine), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::EndOfLine)
            ));
        }
    }
}
