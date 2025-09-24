//! Provides parsers for symbols in the Oneil language.
//!
//! This module contains parsers for all symbol tokens in the Oneil language,
//! including operators, delimiters, and other special characters.

use nom::{
    Parser as _,
    bytes::complete::tag,
    character::complete::{char, satisfy},
    combinator::{eof, peek, value},
};

use crate::token::{
    InputSpan, Parser, Result,
    error::{ExpectSymbol, TokenError},
    util::{Token, token},
};

/// Creates a parser that succeeds when the next character is not the specified character.
///
/// This function is used to prevent partial matches of multi-character symbols.
/// For example, when parsing `=`, we want to ensure the next character is not `=`
/// to avoid matching `==` as two separate `=` tokens.
///
/// The parser succeeds if:
/// - The next character is different from the specified character, OR
/// - We've reached the end of the input
///
/// # Arguments
///
/// * `c` - The character that should NOT be the next character
///
/// # Returns
///
/// A parser that succeeds when the next character is not `c` or at end of input.
fn next_char_is_not<'a>(c: char) -> impl Parser<'a, (), TokenError> {
    let next_char_is_not_c = peek(satisfy(move |next_char: char| next_char != c)).map(|_| ());
    let reached_end_of_file = eof.map(|_| ());
    let mut parser = value((), next_char_is_not_c.or(reached_end_of_file));

    move |input: InputSpan<'a>| parser.parse(input)
}

/// Parses the '!=' symbol token.
pub fn bang_equals(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    token(
        tag("!="),
        TokenError::expected_symbol(ExpectSymbol::BangEquals),
    )
    .parse(input)
}

/// Parses the '|' symbol token.
pub fn bar(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    token(char('|'), TokenError::expected_symbol(ExpectSymbol::Bar)).parse(input)
}

/// Parses the '{' symbol token.
pub fn brace_left(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    token(
        char('{'),
        TokenError::expected_symbol(ExpectSymbol::BraceLeft),
    )
    .parse(input)
}

/// Parses the '[' symbol token.
pub fn bracket_left(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    token(
        char('['),
        TokenError::expected_symbol(ExpectSymbol::BracketLeft),
    )
    .parse(input)
}

/// Parses the ']' symbol token.
pub fn bracket_right(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    token(
        char(']'),
        TokenError::expected_symbol(ExpectSymbol::BracketRight),
    )
    .parse(input)
}

/// Parses the '^' symbol token.
pub fn caret(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    token(char('^'), TokenError::expected_symbol(ExpectSymbol::Caret)).parse(input)
}

/// Parses the ':' symbol token.
pub fn colon(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    token(char(':'), TokenError::expected_symbol(ExpectSymbol::Colon)).parse(input)
}

/// Parses the ',' symbol token.
pub fn comma(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    token(char(','), TokenError::expected_symbol(ExpectSymbol::Comma)).parse(input)
}

/// Parses the '$' symbol token.
pub fn dollar(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    token(char('$'), TokenError::expected_symbol(ExpectSymbol::Dollar)).parse(input)
}

/// Parses the '.' symbol token.
pub fn dot(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    token(
        char('.').and(next_char_is_not('.')),
        TokenError::expected_symbol(ExpectSymbol::Dot),
    )
    .parse(input)
}

/// Parses the '..' symbol token.
pub fn dot_dot(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    token(tag(".."), TokenError::expected_symbol(ExpectSymbol::DotDot)).parse(input)
}

/// Parses the '=' symbol token.
pub fn equals(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    token(
        char('=').and(next_char_is_not('=')),
        TokenError::expected_symbol(ExpectSymbol::Equals),
    )
    .parse(input)
}

/// Parses the '==' symbol token.
pub fn equals_equals(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    token(
        tag("=="),
        TokenError::expected_symbol(ExpectSymbol::EqualsEquals),
    )
    .parse(input)
}

/// Parses the '>' symbol token.
pub fn greater_than(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    token(
        char('>').and(next_char_is_not('=')),
        TokenError::expected_symbol(ExpectSymbol::GreaterThan),
    )
    .parse(input)
}

/// Parses the '>=' symbol token.
pub fn greater_than_equals(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    token(
        tag(">="),
        TokenError::expected_symbol(ExpectSymbol::GreaterThanEquals),
    )
    .parse(input)
}

/// Parses the '<' symbol token.
pub fn less_than(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    token(
        char('<').and(next_char_is_not('=')),
        TokenError::expected_symbol(ExpectSymbol::LessThan),
    )
    .parse(input)
}

/// Parses the '<=' symbol token.
pub fn less_than_equals(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    token(
        tag("<="),
        TokenError::expected_symbol(ExpectSymbol::LessThanEquals),
    )
    .parse(input)
}

/// Parses the '-' symbol token.
pub fn minus(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    token(
        char('-').and(next_char_is_not('-')),
        TokenError::expected_symbol(ExpectSymbol::Minus),
    )
    .parse(input)
}

/// Parses the '--' symbol token.
pub fn minus_minus(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    token(
        tag("--"),
        TokenError::expected_symbol(ExpectSymbol::MinusMinus),
    )
    .parse(input)
}

/// Parses the '(' symbol token.
pub fn paren_left(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    token(
        char('('),
        TokenError::expected_symbol(ExpectSymbol::ParenLeft),
    )
    .parse(input)
}

/// Parses the ')' symbol token.
pub fn paren_right(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    token(
        char(')'),
        TokenError::expected_symbol(ExpectSymbol::ParenRight),
    )
    .parse(input)
}

/// Parses the '%' symbol token.
pub fn percent(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    token(
        char('%'),
        TokenError::expected_symbol(ExpectSymbol::Percent),
    )
    .parse(input)
}

/// Parses the '+' symbol token.
pub fn plus(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    token(char('+'), TokenError::expected_symbol(ExpectSymbol::Plus)).parse(input)
}

/// Parses the '*' symbol token.
pub fn star(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    token(
        char('*').and(next_char_is_not('*')),
        TokenError::expected_symbol(ExpectSymbol::Star),
    )
    .parse(input)
}

/// Parses the '**' symbol token.
pub fn star_star(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    token(
        tag("**"),
        TokenError::expected_symbol(ExpectSymbol::StarStar),
    )
    .parse(input)
}

/// Parses the '/' symbol token.
pub fn slash(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    token(
        char('/').and(next_char_is_not('/')),
        TokenError::expected_symbol(ExpectSymbol::Slash),
    )
    .parse(input)
}

/// Parses the '//' symbol token.
pub fn slash_slash(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    token(
        tag("//"),
        TokenError::expected_symbol(ExpectSymbol::SlashSlash),
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Config, InputSpan, token::error::TokenErrorKind};

    mod success_tests {
        use super::*;

        #[test]
        fn test_bar() {
            let input = InputSpan::new_extra("| rest", Config::default());
            let (rest, matched) = bar(input).expect("should parse '|' symbol");
            assert_eq!(matched.lexeme(), "|");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_brace_left() {
            let input = InputSpan::new_extra("{ rest", Config::default());
            let (rest, matched) = brace_left(input).expect("should parse '{' symbol");
            assert_eq!(matched.lexeme(), "{");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_bracket_left() {
            let input = InputSpan::new_extra("[ rest", Config::default());
            let (rest, matched) = bracket_left(input).expect("should parse '[' symbol");
            assert_eq!(matched.lexeme(), "[");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_bracket_right() {
            let input = InputSpan::new_extra("] rest", Config::default());
            let (rest, matched) = bracket_right(input).expect("should parse ']' symbol");
            assert_eq!(matched.lexeme(), "]");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_caret() {
            let input = InputSpan::new_extra("^ rest", Config::default());
            let (rest, matched) = caret(input).expect("should parse '^' symbol");
            assert_eq!(matched.lexeme(), "^");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_colon() {
            let input = InputSpan::new_extra(": rest", Config::default());
            let (rest, matched) = colon(input).expect("should parse ':' symbol");
            assert_eq!(matched.lexeme(), ":");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_comma() {
            let input = InputSpan::new_extra(", rest", Config::default());
            let (rest, matched) = comma(input).expect("should parse ',' symbol");
            assert_eq!(matched.lexeme(), ",");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_dollar() {
            let input = InputSpan::new_extra("$ rest", Config::default());
            let (rest, matched) = dollar(input).expect("should parse '$' symbol");
            assert_eq!(matched.lexeme(), "$");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_dot() {
            let input = InputSpan::new_extra(". rest", Config::default());
            let (rest, matched) = dot(input).expect("should parse '.' symbol");
            assert_eq!(matched.lexeme(), ".");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_dot_dot() {
            let input = InputSpan::new_extra(".. rest", Config::default());
            let (rest, matched) = dot_dot(input).expect("should parse '..' symbol");
            assert_eq!(matched.lexeme(), "..");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_paren_left() {
            let input = InputSpan::new_extra("( rest", Config::default());
            let (rest, matched) = paren_left(input).expect("should parse '(' symbol");
            assert_eq!(matched.lexeme(), "(");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_paren_right() {
            let input = InputSpan::new_extra(") rest", Config::default());
            let (rest, matched) = paren_right(input).expect("should parse ')' symbol");
            assert_eq!(matched.lexeme(), ")");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_percent() {
            let input = InputSpan::new_extra("% rest", Config::default());
            let (rest, matched) = percent(input).expect("should parse '%' symbol");
            assert_eq!(matched.lexeme(), "%");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_plus() {
            let input = InputSpan::new_extra("+ rest", Config::default());
            let (rest, matched) = plus(input).expect("should parse '+' symbol");
            assert_eq!(matched.lexeme(), "+");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_bang_equals() {
            let input = InputSpan::new_extra("!= rest", Config::default());
            let (rest, matched) = bang_equals(input).expect("should parse '!=' symbol");
            assert_eq!(matched.lexeme(), "!=");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_equals_equals() {
            let input = InputSpan::new_extra("== rest", Config::default());
            let (rest, matched) = equals_equals(input).expect("should parse '==' symbol");
            assert_eq!(matched.lexeme(), "==");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_greater_than_equals() {
            let input = InputSpan::new_extra(">= rest", Config::default());
            let (rest, matched) = greater_than_equals(input).expect("should parse '>=' symbol");
            assert_eq!(matched.lexeme(), ">=");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_less_than_equals() {
            let input = InputSpan::new_extra("<= rest", Config::default());
            let (rest, matched) = less_than_equals(input).expect("should parse '<=' symbol");
            assert_eq!(matched.lexeme(), "<=");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_minus_minus() {
            let input = InputSpan::new_extra("-- rest", Config::default());
            let (rest, matched) = minus_minus(input).expect("should parse '--' symbol");
            assert_eq!(matched.lexeme(), "--");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_star_star() {
            let input = InputSpan::new_extra("** rest", Config::default());
            let (rest, matched) = star_star(input).expect("should parse '**' symbol");
            assert_eq!(matched.lexeme(), "**");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_slash_slash() {
            let input = InputSpan::new_extra("// rest", Config::default());
            let (rest, matched) = slash_slash(input).expect("should parse '//' symbol");
            assert_eq!(matched.lexeme(), "//");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_equals() {
            let input = InputSpan::new_extra("= rest", Config::default());
            let (rest, matched) = equals(input).expect("should parse '=' symbol");
            assert_eq!(matched.lexeme(), "=");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_greater_than() {
            let input = InputSpan::new_extra("> rest", Config::default());
            let (rest, matched) = greater_than(input).expect("should parse '>' symbol");
            assert_eq!(matched.lexeme(), ">");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_less_than() {
            let input = InputSpan::new_extra("< rest", Config::default());
            let (rest, matched) = less_than(input).expect("should parse '<' symbol");
            assert_eq!(matched.lexeme(), "<");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_minus() {
            let input = InputSpan::new_extra("- rest", Config::default());
            let (rest, matched) = minus(input).expect("should parse '-' symbol");
            assert_eq!(matched.lexeme(), "-");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_star() {
            let input = InputSpan::new_extra("* rest", Config::default());
            let (rest, matched) = star(input).expect("should parse '*' symbol");
            assert_eq!(matched.lexeme(), "*");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_slash() {
            let input = InputSpan::new_extra("/ rest", Config::default());
            let (rest, matched) = slash(input).expect("should parse '/' symbol");
            assert_eq!(matched.lexeme(), "/");
            assert_eq!(rest.fragment(), &"rest");
        }

        // Edge cases for all symbol types
        #[test]
        fn test_at_end_of_file() {
            let input = InputSpan::new_extra("+", Config::default());
            let (rest, matched) = plus(input).expect("should parse '+' at end of file");
            assert_eq!(matched.lexeme(), "+");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_multi_char_at_end_of_file() {
            let input = InputSpan::new_extra("!=", Config::default());
            let (rest, matched) = bang_equals(input).expect("should parse '!=' at end of file");
            assert_eq!(matched.lexeme(), "!=");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_with_trailing_whitespace() {
            let input = InputSpan::new_extra("+   rest", Config::default());
            let (rest, matched) = plus(input).expect("should parse '+' with trailing whitespace");
            assert_eq!(matched.lexeme(), "+");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_multi_char_with_trailing_whitespace() {
            let input = InputSpan::new_extra("!=   rest", Config::default());
            let (rest, matched) =
                bang_equals(input).expect("should parse '!=' with trailing whitespace");
            assert_eq!(matched.lexeme(), "!=");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_with_tab() {
            let input = InputSpan::new_extra("+\trest", Config::default());
            let (rest, matched) = plus(input).expect("should parse '+' with tab");
            assert_eq!(matched.lexeme(), "+");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_multi_char_with_tab() {
            let input = InputSpan::new_extra("!=\trest", Config::default());
            let (rest, matched) = bang_equals(input).expect("should parse '!=' with tab");
            assert_eq!(matched.lexeme(), "!=");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_conditional_with_tab() {
            let input = InputSpan::new_extra("=\trest", Config::default());
            let (rest, matched) = equals(input).expect("should parse '=' with tab");
            assert_eq!(matched.lexeme(), "=");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_with_newline() {
            let input = InputSpan::new_extra("+\nrest", Config::default());
            let (rest, matched) = plus(input).expect("should parse '+' with newline");
            assert_eq!(matched.lexeme(), "+");
            assert_eq!(rest.fragment(), &"\nrest");
        }

        #[test]
        fn test_multi_char_with_newline() {
            let input = InputSpan::new_extra("!=\nrest", Config::default());
            let (rest, matched) = bang_equals(input).expect("should parse '!=' with newline");
            assert_eq!(matched.lexeme(), "!=");
            assert_eq!(rest.fragment(), &"\nrest");
        }

        #[test]
        fn test_conditional_with_newline() {
            let input = InputSpan::new_extra("=\nrest", Config::default());
            let (rest, matched) = equals(input).expect("should parse '=' with newline");
            assert_eq!(matched.lexeme(), "=");
            assert_eq!(rest.fragment(), &"\nrest");
        }

        #[test]
        fn test_with_carriage_return() {
            let input = InputSpan::new_extra("+\rrest", Config::default());
            let (rest, matched) = plus(input).expect("should parse '+' with carriage return");
            assert_eq!(matched.lexeme(), "+");
            assert_eq!(rest.fragment(), &"\rrest");
        }
    }

    mod error_tests {
        use crate::token::error::ExpectKind;

        use super::*;

        #[test]
        fn test_empty_input() {
            let input = InputSpan::new_extra("", Config::default());
            let res = plus(input);
            match res {
                Err(nom::Err::Error(token_error)) => {
                    assert!(matches!(
                        token_error.kind,
                        TokenErrorKind::Expect(ExpectKind::Symbol(ExpectSymbol::Plus))
                    ));
                }
                _ => panic!("expected TokenError::Expect(_), got {res:?}"),
            }
        }

        #[test]
        fn test_symbol_not_at_start() {
            let input = InputSpan::new_extra("foo + bar", Config::default());
            let res = plus(input);
            match res {
                Err(nom::Err::Error(token_error)) => {
                    assert!(matches!(
                        token_error.kind,
                        TokenErrorKind::Expect(ExpectKind::Symbol(ExpectSymbol::Plus))
                    ));
                }
                _ => panic!("expected TokenError::Expect(_), got {res:?}"),
            }
        }

        #[test]
        fn test_whitespace_only() {
            let input = InputSpan::new_extra("   ", Config::default());
            let res = plus(input);
            match res {
                Err(nom::Err::Error(token_error)) => {
                    assert!(matches!(
                        token_error.kind,
                        TokenErrorKind::Expect(ExpectKind::Symbol(ExpectSymbol::Plus))
                    ));
                }
                _ => panic!("expected TokenError::Expect(_), got {res:?}"),
            }
        }

        #[test]
        fn test_wrong_symbol() {
            let input = InputSpan::new_extra("x", Config::default());
            let res = plus(input);
            match res {
                Err(nom::Err::Error(token_error)) => {
                    assert!(matches!(
                        token_error.kind,
                        TokenErrorKind::Expect(ExpectKind::Symbol(ExpectSymbol::Plus))
                    ));
                }
                _ => panic!("expected TokenError::Expect(_), got {res:?}"),
            }
        }

        #[test]
        fn test_dot_not_dot_dot() {
            let input = InputSpan::new_extra(".. rest", Config::default());
            let res = dot(input);
            match res {
                Err(nom::Err::Error(token_error)) => {
                    assert!(matches!(
                        token_error.kind,
                        TokenErrorKind::Expect(ExpectKind::Symbol(ExpectSymbol::Dot))
                    ));
                }
                _ => panic!("expected TokenError::Expect(_), got {res:?}"),
            }
        }

        #[test]
        fn test_equals_not_equals_equals() {
            let input = InputSpan::new_extra("== rest", Config::default());
            let res = equals(input);
            match res {
                Err(nom::Err::Error(token_error)) => {
                    assert!(matches!(
                        token_error.kind,
                        TokenErrorKind::Expect(ExpectKind::Symbol(ExpectSymbol::Equals))
                    ));
                }
                _ => panic!("expected TokenError::Expect(_), got {res:?}"),
            }
        }

        #[test]
        fn test_greater_than_not_greater_than_equals() {
            let input = InputSpan::new_extra(">= rest", Config::default());
            let res = greater_than(input);
            match res {
                Err(nom::Err::Error(token_error)) => {
                    assert!(matches!(
                        token_error.kind,
                        TokenErrorKind::Expect(ExpectKind::Symbol(ExpectSymbol::GreaterThan))
                    ));
                }
                _ => panic!("expected TokenError::Expect(_), got {res:?}"),
            }
        }

        #[test]
        fn test_less_than_not_less_than_equals() {
            let input = InputSpan::new_extra("<= rest", Config::default());
            let res = less_than(input);
            match res {
                Err(nom::Err::Error(token_error)) => {
                    assert!(matches!(
                        token_error.kind,
                        TokenErrorKind::Expect(ExpectKind::Symbol(ExpectSymbol::LessThan))
                    ));
                }
                _ => panic!("expected TokenError::Expect(_), got {res:?}"),
            }
        }

        #[test]
        fn test_minus_not_minus_minus() {
            let input = InputSpan::new_extra("-- rest", Config::default());
            let res = minus(input);
            match res {
                Err(nom::Err::Error(token_error)) => {
                    assert!(matches!(
                        token_error.kind,
                        TokenErrorKind::Expect(ExpectKind::Symbol(ExpectSymbol::Minus))
                    ));
                }
                _ => panic!("expected TokenError::Expect(_), got {res:?}"),
            }
        }

        #[test]
        fn test_star_not_star_star() {
            let input = InputSpan::new_extra("** rest", Config::default());
            let res = star(input);
            match res {
                Err(nom::Err::Error(token_error)) => {
                    assert!(matches!(
                        token_error.kind,
                        TokenErrorKind::Expect(ExpectKind::Symbol(ExpectSymbol::Star))
                    ));
                }
                _ => panic!("expected TokenError::Expect(_), got {res:?}"),
            }
        }

        #[test]
        fn test_slash_not_slash_slash() {
            let input = InputSpan::new_extra("// rest", Config::default());
            let res = slash(input);
            match res {
                Err(nom::Err::Error(token_error)) => {
                    assert!(matches!(
                        token_error.kind,
                        TokenErrorKind::Expect(ExpectKind::Symbol(ExpectSymbol::Slash))
                    ));
                }
                _ => panic!("expected TokenError::Expect(_), got {res:?}"),
            }
        }

        #[test]
        fn test_multi_char_symbol_partial() {
            let input = InputSpan::new_extra("!", Config::default());
            let res = bang_equals(input);
            match res {
                Err(nom::Err::Error(token_error)) => {
                    assert!(matches!(
                        token_error.kind,
                        TokenErrorKind::Expect(ExpectKind::Symbol(ExpectSymbol::BangEquals))
                    ));
                }
                _ => panic!("expected TokenError::Expect(_), got {res:?}"),
            }
        }

        #[test]
        fn test_multi_char_symbol_wrong_second_char() {
            let input = InputSpan::new_extra("!x rest", Config::default());
            let res = bang_equals(input);
            match res {
                Err(nom::Err::Error(token_error)) => {
                    assert!(matches!(
                        token_error.kind,
                        TokenErrorKind::Expect(ExpectKind::Symbol(ExpectSymbol::BangEquals))
                    ));
                }
                _ => panic!("expected TokenError::Expect(_), got {res:?}"),
            }
        }
    }

    mod next_char_is_not_tests {
        use super::*;

        #[test]
        fn test_next_char_is_not_succeeds() {
            let input = InputSpan::new_extra("abc", Config::default());
            let (rest, ()) = next_char_is_not('b')
                .parse(input)
                .expect("next char should not be 'b'");
            assert_eq!(rest.fragment(), &"abc");
        }

        #[test]
        fn test_next_char_is_not_succeeds_with_eof() {
            let input = InputSpan::new_extra("", Config::default());
            let (rest, ()) = next_char_is_not('b')
                .parse(input)
                .expect("next char should not be 'b' at EOF");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_next_char_is_not_fails() {
            let input = InputSpan::new_extra("abc", Config::default());
            let res = next_char_is_not('a').parse(input);
            match res {
                Err(nom::Err::Error(token_error)) => {
                    assert!(matches!(token_error.kind, TokenErrorKind::NomError(_)));
                }
                _ => panic!("expected TokenError::NomError(_), got {res:?}"),
            }
        }

        #[test]
        fn test_next_char_is_not_with_whitespace() {
            let input = InputSpan::new_extra(" b", Config::default());
            let (rest, ()) = next_char_is_not('a')
                .parse(input)
                .expect("next char should not be 'a'");
            assert_eq!(rest.fragment(), &" b");
        }

        #[test]
        fn test_next_char_is_not_with_special_characters() {
            let input = InputSpan::new_extra("!bc", Config::default());
            let (rest, ()) = next_char_is_not('a')
                .parse(input)
                .expect("next char should not be 'a'");
            assert_eq!(rest.fragment(), &"!bc");
        }

        #[test]
        fn test_next_char_is_not_with_numbers() {
            let input = InputSpan::new_extra("123", Config::default());
            let (rest, ()) = next_char_is_not('a')
                .parse(input)
                .expect("next char should not be 'a'");
            assert_eq!(rest.fragment(), &"123");
        }

        #[test]
        fn test_next_char_is_not_with_unicode() {
            let input = InputSpan::new_extra("αβγ", Config::default());
            let (rest, ()) = next_char_is_not('a')
                .parse(input)
                .expect("next char should not be 'a'");
            assert_eq!(rest.fragment(), &"αβγ");
        }

        #[test]
        fn test_next_char_is_not_with_emoji() {
            let input = InputSpan::new_extra("🚀bc", Config::default());
            let (rest, ()) = next_char_is_not('a')
                .parse(input)
                .expect("next char should not be 'a'");
            assert_eq!(rest.fragment(), &"🚀bc");
        }
    }
}
