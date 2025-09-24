//! Provides parsers for Oneil language keywords.
//!
//! This module contains parsers for all reserved keywords in the Oneil language.
//! Each keyword parser ensures that the keyword is followed by a non-alphanumeric
//! character or '_' to prevent partial matches (e.g., "and" vs "android").

use nom::{
    Parser as _,
    bytes::complete::tag,
    character::complete::satisfy,
    combinator::{eof, peek},
};

use crate::token::{
    InputSpan, Parser, Result,
    error::{self, TokenError},
    util::{Token, token},
};

/// Keywords that are valid in the Oneil language.
pub const KEYWORDS: &[&str] = &[
    "and", "as", "false", "from", "if", "import", "not", "or", "ref", "section", "test", "true",
    "use", "with",
];

/// Creates a keyword parser for the specified keyword string.
///
/// This function constructs a parser that matches the exact keyword string
/// and ensures it's not followed by an alphanumeric character or underscore.
/// This prevents partial matches (e.g., "and" vs "android").
///
/// The parser succeeds if:
/// - The input starts with the exact keyword string
/// - The keyword is followed by a non-alphanumeric character (or end of input)
///
/// # Arguments
///
/// * `kw_str` - The keyword string to match
/// * `error_kind` - The error kind to use if the keyword is not found
///
/// # Returns
///
/// A parser that matches the specified keyword with proper boundary checking.
fn keyword(
    kw_str: &str,
    error_kind: error::ExpectKeyword,
) -> impl Parser<'_, Token<'_>, error::TokenError> {
    token(
        move |input| {
            let next_char_is_not_ident_char =
                peek(satisfy(|c: char| !c.is_alphanumeric() && c != '_')).map(|_| ());

            let reached_end_of_file = eof.map(|_| ());

            let (input, _) = tag(kw_str)(input)?;
            let (input, ()) = next_char_is_not_ident_char
                .or(reached_end_of_file)
                .parse(input)?;
            Ok((input, ()))
        },
        TokenError::expected_keyword(error_kind),
    )
}

/// Parses the 'and' keyword token.
pub fn and(input: InputSpan<'_>) -> Result<'_, Token<'_>, error::TokenError> {
    keyword("and", error::ExpectKeyword::And).parse(input)
}

/// Parses the 'as' keyword token.
pub fn as_(input: InputSpan<'_>) -> Result<'_, Token<'_>, error::TokenError> {
    keyword("as", error::ExpectKeyword::As).parse(input)
}

/// Parses the 'false' keyword token.
pub fn false_(input: InputSpan<'_>) -> Result<'_, Token<'_>, error::TokenError> {
    keyword("false", error::ExpectKeyword::False).parse(input)
}

/// Parses the 'if' keyword token.
pub fn if_(input: InputSpan<'_>) -> Result<'_, Token<'_>, error::TokenError> {
    keyword("if", error::ExpectKeyword::If).parse(input)
}

/// Parses the 'import' keyword token.
pub fn import(input: InputSpan<'_>) -> Result<'_, Token<'_>, error::TokenError> {
    keyword("import", error::ExpectKeyword::Import).parse(input)
}

/// Parses the 'not' keyword token.
pub fn not(input: InputSpan<'_>) -> Result<'_, Token<'_>, error::TokenError> {
    keyword("not", error::ExpectKeyword::Not).parse(input)
}

/// Parses the 'or' keyword token.
pub fn or(input: InputSpan<'_>) -> Result<'_, Token<'_>, error::TokenError> {
    keyword("or", error::ExpectKeyword::Or).parse(input)
}

/// Parses the 'ref' keyword token.
pub fn ref_(input: InputSpan<'_>) -> Result<'_, Token<'_>, error::TokenError> {
    keyword("ref", error::ExpectKeyword::Ref).parse(input)
}

/// Parses the 'section' keyword token.
pub fn section(input: InputSpan<'_>) -> Result<'_, Token<'_>, error::TokenError> {
    keyword("section", error::ExpectKeyword::Section).parse(input)
}

/// Parses the 'test' keyword token.
pub fn test(input: InputSpan<'_>) -> Result<'_, Token<'_>, error::TokenError> {
    keyword("test", error::ExpectKeyword::Test).parse(input)
}

/// Parses the 'true' keyword token.
pub fn true_(input: InputSpan<'_>) -> Result<'_, Token<'_>, error::TokenError> {
    keyword("true", error::ExpectKeyword::True).parse(input)
}

/// Parses the 'use' keyword token.
pub fn use_(input: InputSpan<'_>) -> Result<'_, Token<'_>, error::TokenError> {
    keyword("use", error::ExpectKeyword::Use).parse(input)
}

/// Parses the 'with' keyword token.
pub fn with(input: InputSpan<'_>) -> Result<'_, Token<'_>, error::TokenError> {
    keyword("with", error::ExpectKeyword::With).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Config, InputSpan,
        token::error::{ExpectKind, TokenErrorKind},
    };

    mod success_tests {
        use super::*;

        #[test]
        fn test_and() {
            let input = InputSpan::new_extra("and rest", Config::default());
            let (rest, matched) = and(input).expect("should parse 'and' keyword");
            assert_eq!(matched.lexeme(), "and");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_as() {
            let input = InputSpan::new_extra("as foo", Config::default());
            let (rest, matched) = as_(input).expect("should parse 'as' keyword");
            assert_eq!(matched.lexeme(), "as");
            assert_eq!(rest.fragment(), &"foo");
        }

        #[test]
        fn test_false() {
            let input = InputSpan::new_extra("false true", Config::default());
            let (rest, matched) = false_(input).expect("should parse 'false' keyword");
            assert_eq!(matched.lexeme(), "false");
            assert_eq!(rest.fragment(), &"true");
        }

        #[test]
        fn test_if() {
            let input = InputSpan::new_extra("if baz", Config::default());
            let (rest, matched) = if_(input).expect("should parse 'if' keyword");
            assert_eq!(matched.lexeme(), "if");
            assert_eq!(rest.fragment(), &"baz");
        }

        #[test]
        fn test_import() {
            let input = InputSpan::new_extra("import foo", Config::default());
            let (rest, matched) = import(input).expect("should parse 'import' keyword");
            assert_eq!(matched.lexeme(), "import");
            assert_eq!(rest.fragment(), &"foo");
        }

        #[test]
        fn test_not() {
            let input = InputSpan::new_extra("not bar", Config::default());
            let (rest, matched) = not(input).expect("should parse 'not' keyword");
            assert_eq!(matched.lexeme(), "not");
            assert_eq!(rest.fragment(), &"bar");
        }

        #[test]
        fn test_or() {
            let input = InputSpan::new_extra("or baz", Config::default());
            let (rest, matched) = or(input).expect("should parse 'or' keyword");
            assert_eq!(matched.lexeme(), "or");
            assert_eq!(rest.fragment(), &"baz");
        }

        #[test]
        fn test_ref() {
            let input = InputSpan::new_extra("ref foo", Config::default());
            let (rest, matched) = ref_(input).expect("should parse 'ref' keyword");
            assert_eq!(matched.lexeme(), "ref");
            assert_eq!(rest.fragment(), &"foo");
        }

        #[test]
        fn test_section() {
            let input = InputSpan::new_extra("section test", Config::default());
            let (rest, matched) = section(input).expect("should parse 'section' keyword");
            assert_eq!(matched.lexeme(), "section");
            assert_eq!(rest.fragment(), &"test");
        }

        #[test]
        fn test_test() {
            let input = InputSpan::new_extra("test use", Config::default());
            let (rest, matched) = test(input).expect("should parse 'test' keyword");
            assert_eq!(matched.lexeme(), "test");
            assert_eq!(rest.fragment(), &"use");
        }

        #[test]
        fn test_true() {
            let input = InputSpan::new_extra("true false", Config::default());
            let (rest, matched) = true_(input).expect("should parse 'true' keyword");
            assert_eq!(matched.lexeme(), "true");
            assert_eq!(rest.fragment(), &"false");
        }

        #[test]
        fn test_use() {
            let input = InputSpan::new_extra("use foo", Config::default());
            let (rest, matched) = use_(input).expect("should parse 'use' keyword");
            assert_eq!(matched.lexeme(), "use");
            assert_eq!(rest.fragment(), &"foo");
        }

        #[test]
        fn test_with() {
            let input = InputSpan::new_extra("with foo", Config::default());
            let (rest, matched) = with(input).expect("should parse 'with' keyword");
            assert_eq!(matched.lexeme(), "with");
            assert_eq!(rest.fragment(), &"foo");
        }

        #[test]
        fn test_with_trailing_whitespace() {
            let input = InputSpan::new_extra("and   foo", Config::default());
            let (rest, matched) = and(input).expect("should parse 'and' with trailing whitespace");
            assert_eq!(matched.lexeme(), "and");
            assert_eq!(rest.fragment(), &"foo");
        }

        #[test]
        fn test_at_end_of_file() {
            let input = InputSpan::new_extra("and", Config::default());
            let (rest, matched) = and(input).expect("should parse 'and' at end of file");
            assert_eq!(matched.lexeme(), "and");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_with_punctuation() {
            let input = InputSpan::new_extra("and,", Config::default());
            let (rest, matched) = and(input).expect("should parse 'and' with comma");
            assert_eq!(matched.lexeme(), "and");
            assert_eq!(rest.fragment(), &",");
        }

        #[test]
        fn test_with_parentheses() {
            let input = InputSpan::new_extra("if(", Config::default());
            let (rest, matched) = if_(input).expect("should parse 'if' with opening parenthesis");
            assert_eq!(matched.lexeme(), "if");
            assert_eq!(rest.fragment(), &"(");
        }

        #[test]
        fn test_with_underscore() {
            let input = InputSpan::new_extra("import_", Config::default());
            let res = import(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Keyword(error::ExpectKeyword::Import))
                )),
                _ => panic!("expected TokenError::Expect(Keyword(Import)), got {res:?}"),
            }
        }

        #[test]
        fn test_with_symbols() {
            let input = InputSpan::new_extra("not+", Config::default());
            let (rest, matched) = not(input).expect("should parse 'not' with plus symbol");
            assert_eq!(matched.lexeme(), "not");
            assert_eq!(rest.fragment(), &"+");
        }

        #[test]
        fn test_with_newline() {
            let input = InputSpan::new_extra("true\n", Config::default());
            let (rest, matched) = true_(input).expect("should parse 'true' with newline");
            assert_eq!(matched.lexeme(), "true");
            assert_eq!(rest.fragment(), &"\n");
        }

        #[test]
        fn test_with_tab() {
            let input = InputSpan::new_extra("import\t", Config::default());
            let (rest, matched) = import(input).expect("should parse 'import' with tab");
            assert_eq!(matched.lexeme(), "import");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_with_carriage_return() {
            let input = InputSpan::new_extra("section\r", Config::default());
            let (rest, matched) =
                section(input).expect("should parse 'section' with carriage return");
            assert_eq!(matched.lexeme(), "section");
            assert_eq!(rest.fragment(), &"\r");
        }
    }

    mod error_tests {
        use super::*;

        #[test]
        fn test_empty_input() {
            let input = InputSpan::new_extra("", Config::default());
            let res = and(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Keyword(error::ExpectKeyword::And))
                )),
                _ => panic!("expected TokenError::Expect(Keyword(And)), got {res:?}"),
            }
        }

        #[test]
        fn test_whitespace_only() {
            let input = InputSpan::new_extra("   ", Config::default());
            let res = and(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Keyword(error::ExpectKeyword::And))
                )),
                _ => panic!("expected TokenError::Expect(Keyword(And)), got {res:?}"),
            }
        }

        #[test]
        fn test_wrong_keyword() {
            let input = InputSpan::new_extra("or", Config::default());
            let res = and(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Keyword(error::ExpectKeyword::And))
                )),
                _ => panic!("expected TokenError::Expect(Keyword(And)), got {res:?}"),
            }
        }

        #[test]
        fn test_partial_match() {
            let input = InputSpan::new_extra("andrew", Config::default());
            let res = and(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Keyword(error::ExpectKeyword::And))
                )),
                _ => panic!("expected TokenError::Expect(Keyword(And)), got {res:?}"),
            }
        }

        #[test]
        fn test_prefix_match() {
            let input = InputSpan::new_extra("android", Config::default());
            let res = and(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Keyword(error::ExpectKeyword::And))
                )),
                _ => panic!("expected TokenError::Expect(Keyword(And)), got {res:?}"),
            }
        }

        #[test]
        fn test_with_letters_after() {
            let input = InputSpan::new_extra("andx", Config::default());
            let res = and(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Keyword(error::ExpectKeyword::And))
                )),
                _ => panic!("expected TokenError::Expect(Keyword(And)), got {res:?}"),
            }
        }

        #[test]
        fn test_with_numbers_after() {
            let input = InputSpan::new_extra("and123", Config::default());
            let res = and(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Keyword(error::ExpectKeyword::And))
                )),
                _ => panic!("expected TokenError::Expect(Keyword(And)), got {res:?}"),
            }
        }

        #[test]
        fn test_with_underscore_after() {
            let input = InputSpan::new_extra("and_", Config::default());
            let res = and(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Keyword(error::ExpectKeyword::And))
                )),
                _ => panic!("expected TokenError::Expect(Keyword(And)), got {res:?}"),
            }
        }

        #[test]
        fn test_not_at_start() {
            let input = InputSpan::new_extra("foo and bar", Config::default());
            let res = and(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Keyword(error::ExpectKeyword::And))
                )),
                _ => panic!("expected TokenError::Expect(Keyword(And)), got {res:?}"),
            }
        }

        #[test]
        fn test_case_sensitive() {
            let input = InputSpan::new_extra("AND", Config::default());
            let res = and(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Keyword(error::ExpectKeyword::And))
                )),
                _ => panic!("expected TokenError::Expect(Keyword(And)), got {res:?}"),
            }
        }

        #[test]
        fn test_mixed_case() {
            let input = InputSpan::new_extra("And", Config::default());
            let res = and(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Keyword(error::ExpectKeyword::And))
                )),
                _ => panic!("expected TokenError::Expect(Keyword(And)), got {res:?}"),
            }
        }

        #[test]
        fn test_special_characters() {
            let input = InputSpan::new_extra("!@#", Config::default());
            let res = and(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Keyword(error::ExpectKeyword::And))
                )),
                _ => panic!("expected TokenError::Expect(Keyword(And)), got {res:?}"),
            }
        }

        #[test]
        fn test_numbers_only() {
            let input = InputSpan::new_extra("123", Config::default());
            let res = and(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Keyword(error::ExpectKeyword::And))
                )),
                _ => panic!("expected TokenError::Expect(Keyword(And)), got {res:?}"),
            }
        }

        #[test]
        fn test_symbols_only() {
            let input = InputSpan::new_extra("+-*/", Config::default());
            let res = and(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Keyword(error::ExpectKeyword::And))
                )),
                _ => panic!("expected TokenError::Expect(Keyword(And)), got {res:?}"),
            }
        }

        // Test all keywords with error cases
        #[test]
        fn test_as_error() {
            let input = InputSpan::new_extra("wrong", Config::default());
            let res = as_(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Keyword(error::ExpectKeyword::As))
                )),
                _ => panic!("expected TokenError::Expect(Keyword(As)), got {res:?}"),
            }
        }

        #[test]
        fn test_false_error() {
            let input = InputSpan::new_extra("wrong", Config::default());
            let res = false_(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Keyword(error::ExpectKeyword::False))
                )),
                _ => panic!("expected TokenError::Expect(Keyword(False)), got {res:?}"),
            }
        }

        #[test]
        fn test_if_error() {
            let input = InputSpan::new_extra("wrong", Config::default());
            let res = if_(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Keyword(error::ExpectKeyword::If))
                )),
                _ => panic!("expected TokenError::Expect(Keyword(If)), got {res:?}"),
            }
        }

        #[test]
        fn test_import_error() {
            let input = InputSpan::new_extra("wrong", Config::default());
            let res = import(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Keyword(error::ExpectKeyword::Import))
                )),
                _ => panic!("expected TokenError::Expect(Keyword(Import)), got {res:?}"),
            }
        }

        #[test]
        fn test_not_error() {
            let input = InputSpan::new_extra("wrong", Config::default());
            let res = not(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Keyword(error::ExpectKeyword::Not))
                )),
                _ => panic!("expected TokenError::Expect(Keyword(Not)), got {res:?}"),
            }
        }

        #[test]
        fn test_or_error() {
            let input = InputSpan::new_extra("wrong", Config::default());
            let res = or(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Keyword(error::ExpectKeyword::Or))
                )),
                _ => panic!("expected TokenError::Expect(Keyword(Or)), got {res:?}"),
            }
        }

        #[test]
        fn test_section_error() {
            let input = InputSpan::new_extra("wrong", Config::default());
            let res = section(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Keyword(error::ExpectKeyword::Section))
                )),
                _ => panic!("expected TokenError::Expect(Keyword(Section)), got {res:?}"),
            }
        }

        #[test]
        fn test_ref_error() {
            let input = InputSpan::new_extra("wrong", Config::default());
            let res = ref_(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Keyword(error::ExpectKeyword::Ref))
                )),
                _ => panic!("expected TokenError::Expect(Keyword(Ref)), got {res:?}"),
            }
        }

        #[test]
        fn test_test_error() {
            let input = InputSpan::new_extra("wrong", Config::default());
            let res = test(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Keyword(error::ExpectKeyword::Test))
                )),
                _ => panic!("expected TokenError::Expect(Keyword(Test)), got {res:?}"),
            }
        }

        #[test]
        fn test_true_error() {
            let input = InputSpan::new_extra("wrong", Config::default());
            let res = true_(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Keyword(error::ExpectKeyword::True))
                )),
                _ => panic!("expected TokenError::Expect(Keyword(True)), got {res:?}"),
            }
        }

        #[test]
        fn test_use_error() {
            let input = InputSpan::new_extra("wrong", Config::default());
            let res = use_(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Keyword(error::ExpectKeyword::Use))
                )),
                _ => panic!("expected TokenError::Expect(Keyword(Use)), got {res:?}"),
            }
        }

        #[test]
        fn test_with_error() {
            let input = InputSpan::new_extra("wrong", Config::default());
            let res = with(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Keyword(error::ExpectKeyword::With))
                )),
                _ => panic!("expected TokenError::Expect(Keyword(With)), got {res:?}"),
            }
        }

        #[test]
        fn test_error_messages_are_specific() {
            let input = InputSpan::new_extra("abc", Config::default());
            let res = and(input);
            assert!(res.is_err(), "should fail with specific error");

            if let Err(nom::Err::Error(token_error)) = res {
                assert!(
                    matches!(
                        token_error.kind,
                        TokenErrorKind::Expect(ExpectKind::Keyword(error::ExpectKeyword::And))
                    ),
                    "error should be for And keyword"
                );
            } else {
                panic!("expected TokenError but got different error type: {res:?}");
            }
        }
    }

    mod keyword_constants_tests {
        use super::*;

        #[test]
        fn test_keywords_constant_contains_no_duplicates() {
            let mut sorted = KEYWORDS.to_vec();
            sorted.sort_unstable();
            let mut deduped = sorted.clone();
            deduped.dedup();
            assert_eq!(
                sorted.len(),
                deduped.len(),
                "KEYWORDS should contain no duplicates"
            );
        }
    }
}
