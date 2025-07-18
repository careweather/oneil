//! Provides parsers for identifiers and labels in the Oneil language.
//!
//! This module contains parsers for identifiers (variable names, function names, etc.)
//! and labels (section names, test names, etc.). Identifiers follow standard programming
//! language rules, while labels are more permissive to allow for descriptive names.

use nom::{
    Parser as _,
    bytes::complete::{take_while, take_while1},
    character::complete::satisfy,
    combinator::verify,
    multi::many0,
};

use crate::token::{
    Result, Span,
    error::TokenError,
    keyword,
    util::{Token, token},
};

/// Parses an identifier (alphabetic or underscore, then alphanumeric or underscore).
///
/// Identifiers in Oneil follow standard programming language rules:
/// - Must start with an alphabetic character or underscore
/// - Can be followed by any number of alphanumeric characters or underscores
/// - Cannot be a reserved keyword
///
/// Examples of valid identifiers:
/// - `foo`, `bar`, `baz`
/// - `_private`, `public_var`
/// - `user123`, `test_case`
/// - `camelCase`, `snake_case`
///
/// Examples of invalid identifiers:
/// - `123abc` (starts with digit)
/// - `my-var` (contains dash)
/// - `if` (reserved keyword)
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns a token containing the parsed identifier, or an error if the input
/// is not a valid identifier or is a reserved keyword.
pub fn identifier(input: Span) -> Result<Token, TokenError> {
    verify(
        token(
            |input| {
                // First character must be alphabetic or underscore
                let (rest, _) = satisfy(|c: char| c.is_alphabetic() || c == '_').parse(input)?;
                // Remaining characters can be alphanumeric or underscore
                let (rest, _) =
                    take_while(|c: char| c.is_alphanumeric() || c == '_').parse(rest)?;
                Ok((rest, ()))
            },
            TokenError::expected_identifier,
        ),
        // Ensure the identifier is not a reserved keyword
        |identifier| !keyword::KEYWORDS.contains(&identifier.lexeme()),
    )
    .parse(input)
}

/// Parses a label (like an identifier but can contain spaces, tabs, and dashes).
///
/// Labels in Oneil are more permissive than identifiers to allow for descriptive names.
/// They are commonly used for section names, test names, and other human-readable labels.
///
/// Label syntax:
/// - Must start with an alphanumeric character or underscore
/// - Can contain alphanumeric characters, underscores, dashes, and apostrophes
/// - Can contain spaces and tabs between word parts
/// - Cannot be a reserved keyword
///
/// Examples of valid labels:
/// - `foo-bar`, `my-section`
/// - `foo bar`, `test case`
/// - `foo\tbar` (with tab)
/// - `user's data`, `test-123`
/// - `foo bar baz`, `section-name with spaces`
///
/// Examples of invalid labels:
/// - `-foo` (starts with dash)
/// - `if` (reserved keyword)
/// - `123test` (starts with digit)
///
/// Note that labels are often followed by a colon as a delimiter, but other
/// tokens (such as a linebreak) can also be used depending on the context.
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns a token containing the parsed label, or an error if the input
/// is not a valid label or is a reserved keyword.
pub fn label(input: Span) -> Result<Token, TokenError> {
    verify(
        token(
            |input| {
                // First character must be alphanumeric or underscore
                let (rest, _) = satisfy(|c: char| c.is_alphanumeric() || c == '_').parse(input)?;

                // Parse zero or more word parts separated by whitespace
                let (rest, _) = many0(|input| {
                    // Consume optional whitespace (spaces and tabs)
                    let (rest, _) = take_while(|c: char| c == ' ' || c == '\t').parse(input)?;
                    // Consume at least one word character (alphanumeric, underscore, dash, or apostrophe)
                    let (rest, _) = take_while1(|c: char| {
                        c.is_alphanumeric() || c == '_' || c == '-' || c == '\''
                    })
                    .parse(rest)?;
                    Ok((rest, ()))
                })
                .parse(rest)?;

                Ok((rest, ()))
            },
            TokenError::expected_label,
        ),
        // Ensure the label is not a reserved keyword
        |label| !keyword::KEYWORDS.contains(&label.lexeme()),
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::error::{ExpectKind, TokenErrorKind};
    use crate::{Config, Span};

    mod identifier_tests {
        use super::*;

        #[test]
        fn test_basic() {
            let input = Span::new_extra("foo rest", Config::default());
            let (rest, matched) = identifier(input).expect("should parse basic identifier");
            assert_eq!(matched.lexeme(), "foo");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_underscore() {
            let input = Span::new_extra("_foo123 bar", Config::default());
            let (rest, matched) =
                identifier(input).expect("should parse identifier with underscore");
            assert_eq!(matched.lexeme(), "_foo123");
            assert_eq!(rest.fragment(), &"bar");
        }

        #[test]
        fn test_only_underscore() {
            let input = Span::new_extra("_ rest", Config::default());
            let (rest, matched) =
                identifier(input).expect("should parse single underscore identifier");
            assert_eq!(matched.lexeme(), "_");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_camel_case() {
            let input = Span::new_extra("camelCase rest", Config::default());
            let (rest, matched) = identifier(input).expect("should parse camelCase identifier");
            assert_eq!(matched.lexeme(), "camelCase");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_snake_case() {
            let input = Span::new_extra("snake_case rest", Config::default());
            let (rest, matched) = identifier(input).expect("should parse snake_case identifier");
            assert_eq!(matched.lexeme(), "snake_case");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_with_numbers() {
            let input = Span::new_extra("user123 rest", Config::default());
            let (rest, matched) = identifier(input).expect("should parse identifier with numbers");
            assert_eq!(matched.lexeme(), "user123");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_with_trailing_whitespace() {
            let input = Span::new_extra("foo rest", Config::default());
            let (rest, matched) =
                identifier(input).expect("should parse identifier with trailing whitespace");
            assert_eq!(matched.lexeme(), "foo");
            assert_eq!(matched.whitespace(), " ");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_empty_input() {
            let input = Span::new_extra("", Config::default());
            let res = identifier(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Identifier)
                )),
                _ => panic!("expected TokenError::Expect(Identifier), got {:?}", res),
            }
        }

        #[test]
        fn test_starts_with_digit() {
            let input = Span::new_extra("123abc", Config::default());
            let res = identifier(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Identifier)
                )),
                _ => panic!("expected TokenError::Expect(Identifier), got {:?}", res),
            }
        }

        #[test]
        fn test_starts_with_dash() {
            let input = Span::new_extra("-foo", Config::default());
            let res = identifier(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Identifier)
                )),
                _ => panic!("expected TokenError::Expect(Identifier), got {:?}", res),
            }
        }

        #[test]
        fn test_followed_by_dash() {
            let input = Span::new_extra("foo-bar", Config::default());
            let (rest, matched) =
                identifier(input).expect("should parse identifier followed by dash");
            assert_eq!(matched.lexeme(), "foo");
            assert_eq!(rest.fragment(), &"-bar");
        }

        #[test]
        fn test_followed_by_space() {
            let input = Span::new_extra("foo bar", Config::default());
            let (rest, matched) =
                identifier(input).expect("should parse identifier followed by space");
            assert_eq!(matched.lexeme(), "foo");
            assert_eq!(rest.fragment(), &"bar");
        }

        #[test]
        fn test_followed_by_special_characters() {
            let input = Span::new_extra("foo@bar", Config::default());
            let (rest, matched) =
                identifier(input).expect("should parse identifier followed by special characters");
            assert_eq!(matched.lexeme(), "foo");
            assert_eq!(rest.fragment(), &"@bar");
        }

        #[test]
        fn test_keyword_fails() {
            for keyword in keyword::KEYWORDS {
                let input = Span::new_extra(keyword, Config::default());
                let res = identifier(input);
                match res {
                    Err(nom::Err::Error(token_error)) => {
                        // The verify function fails with NomError(Verify) when a keyword is provided
                        assert!(matches!(
                            token_error.kind,
                            TokenErrorKind::NomError(nom::error::ErrorKind::Verify)
                        ));
                    }
                    _ => panic!(
                        "expected TokenError::NomError(Verify) for keyword {:?}, got {:?}",
                        keyword, res
                    ),
                }
            }
        }

        #[test]
        fn test_starts_with_keyword() {
            let input = Span::new_extra("iffoo", Config::default());
            let (rest, matched) =
                identifier(input).expect("should parse identifier starting with keyword");
            assert_eq!(matched.lexeme(), "iffoo");
            assert_eq!(rest.fragment(), &"");
        }
    }

    mod label_tests {
        use super::*;

        #[test]
        fn test_basic() {
            let input = Span::new_extra("foo-bar: rest", Config::default());
            let (rest, matched) = label(input).expect("should parse label with dash");
            assert_eq!(matched.lexeme(), "foo-bar");
            assert_eq!(rest.fragment(), &": rest");
        }

        #[test]
        fn test_with_spaces_and_tabs() {
            let input = Span::new_extra("foo bar\tbaz: rest", Config::default());
            let (rest, matched) = label(input).expect("should parse label with spaces and tabs");
            assert_eq!(matched.lexeme(), "foo bar\tbaz");
            assert_eq!(rest.fragment(), &": rest");
        }

        #[test]
        fn test_with_numbers() {
            let input = Span::new_extra("123Test: rest", Config::default());
            let (rest, matched) = label(input).expect("should parse label with numbers");
            assert_eq!(matched.lexeme(), "123Test");
            assert_eq!(rest.fragment(), &": rest");
        }

        #[test]
        fn test_only_underscore() {
            let input = Span::new_extra("_: rest", Config::default());
            let (rest, matched) = label(input).expect("should parse label with only underscore");
            assert_eq!(matched.lexeme(), "_");
            assert_eq!(rest.fragment(), &": rest");
        }

        #[test]
        fn test_with_multiple_dashes() {
            let input = Span::new_extra("foo-bar-baz: rest", Config::default());
            let (rest, matched) = label(input).expect("should parse label with multiple dashes");
            assert_eq!(matched.lexeme(), "foo-bar-baz");
            assert_eq!(rest.fragment(), &": rest");
        }

        #[test]
        fn test_with_trailing_whitespace() {
            let input = Span::new_extra("foo : rest", Config::default());
            let (rest, matched) =
                label(input).expect("should parse label with trailing whitespace");
            assert_eq!(matched.lexeme(), "foo");
            assert_eq!(matched.whitespace(), " ");
            assert_eq!(rest.fragment(), &": rest");
        }

        #[test]
        fn test_with_multiple_spaces() {
            let input = Span::new_extra("foo bar : rest", Config::default());
            let (rest, matched) = label(input).expect("should parse label with multiple spaces");
            assert_eq!(matched.lexeme(), "foo bar");
            assert_eq!(matched.whitespace(), " ");
            assert_eq!(rest.fragment(), &": rest");
        }

        #[test]
        fn test_with_apostrophe() {
            let input = Span::new_extra("user's data: rest", Config::default());
            let (rest, matched) = label(input).expect("should parse label with apostrophe");
            assert_eq!(matched.lexeme(), "user's data");
            assert_eq!(rest.fragment(), &": rest");
        }

        #[test]
        fn test_with_multiple_words() {
            let input = Span::new_extra("section name with spaces: rest", Config::default());
            let (rest, matched) = label(input).expect("should parse label with multiple words");
            assert_eq!(matched.lexeme(), "section name with spaces");
            assert_eq!(rest.fragment(), &": rest");
        }

        #[test]
        fn test_with_mixed_separators() {
            let input = Span::new_extra("test-case with spaces: rest", Config::default());
            let (rest, matched) = label(input).expect("should parse label with mixed separators");
            assert_eq!(matched.lexeme(), "test-case with spaces");
            assert_eq!(rest.fragment(), &": rest");
        }

        #[test]
        fn test_empty_input() {
            let input = Span::new_extra("", Config::default());
            let res = label(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Label)
                )),
                _ => panic!("expected TokenError::Expect(Label), got {:?}", res),
            }
        }

        #[test]
        fn test_invalid_start() {
            let input = Span::new_extra("-foo", Config::default());
            let res = label(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Label)
                )),
                _ => panic!("expected TokenError::Expect(Label), got {:?}", res),
            }
        }

        #[test]
        fn test_followed_by_special_characters() {
            let input = Span::new_extra("foo@bar", Config::default());
            let (rest, matched) =
                label(input).expect("should parse label followed by special characters");
            assert_eq!(matched.lexeme(), "foo");
            assert_eq!(rest.fragment(), &"@bar");
        }

        #[test]
        fn test_followed_by_newline() {
            let input = Span::new_extra("foo\nbar", Config::default());
            let (rest, matched) = label(input).expect("should parse label followed by newline");
            assert_eq!(matched.lexeme(), "foo");
            assert_eq!(rest.fragment(), &"\nbar");
        }

        #[test]
        fn test_followed_by_carriage_return() {
            let input = Span::new_extra("foo\rbar", Config::default());
            let (rest, matched) =
                label(input).expect("should parse label followed by carriage return");
            assert_eq!(matched.lexeme(), "foo");
            assert_eq!(rest.fragment(), &"\rbar");
        }

        #[test]
        fn test_keyword_fails() {
            for keyword in keyword::KEYWORDS {
                let input = Span::new_extra(keyword, Config::default());
                let res = label(input);
                match res {
                    Err(nom::Err::Error(token_error)) => {
                        // The verify function fails with NomError(Verify) when a keyword is provided
                        assert!(matches!(
                            token_error.kind,
                            TokenErrorKind::NomError(nom::error::ErrorKind::Verify)
                        ));
                    }
                    _ => panic!(
                        "expected TokenError::NomError(Verify) for keyword {:?}, got {:?}",
                        keyword, res
                    ),
                }
            }
        }

        #[test]
        fn test_starts_with_keyword() {
            let input = Span::new_extra("ifelse", Config::default());
            let (rest, matched) = label(input).expect("should parse label starting with keyword");
            assert_eq!(matched.lexeme(), "ifelse");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_contains_keyword_multiple_words() {
            let input = Span::new_extra("if foo test", Config::default());
            let (rest, matched) =
                label(input).expect("should parse label containing keyword and multiple words");
            assert_eq!(matched.lexeme(), "if foo test");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_only_whitespace() {
            let input = Span::new_extra("   ", Config::default());
            let res = label(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Label)
                )),
                _ => panic!("expected TokenError::Expect(Label), got {:?}", res),
            }
        }

        #[test]
        fn test_whitespace_after_word() {
            let input = Span::new_extra("foo   ", Config::default());
            let (rest, matched) =
                label(input).expect("should parse label with trailing whitespace");
            assert_eq!(matched.lexeme(), "foo");
            assert_eq!(matched.whitespace(), "   ");
            assert_eq!(rest.fragment(), &"");
        }
    }
}
