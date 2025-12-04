//! Provides parsers for symbols in the Oneil language.

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
    token(char('-'), TokenError::expected_symbol(ExpectSymbol::Minus)).parse(input)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Config, InputSpan, token::error::TokenErrorKind};

    mod success {
        use super::*;

        #[test]
        fn bar_symbol() {
            let input = InputSpan::new_extra("| rest", Config::default());
            let (rest, matched) = bar(input).expect("should parse '|' symbol");
            assert_eq!(matched.lexeme_str, "|");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn brace_left_symbol() {
            let input = InputSpan::new_extra("{ rest", Config::default());
            let (rest, matched) = brace_left(input).expect("should parse '{' symbol");
            assert_eq!(matched.lexeme_str, "{");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn bracket_left_symbol() {
            let input = InputSpan::new_extra("[ rest", Config::default());
            let (rest, matched) = bracket_left(input).expect("should parse '[' symbol");
            assert_eq!(matched.lexeme_str, "[");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn bracket_right_symbol() {
            let input = InputSpan::new_extra("] rest", Config::default());
            let (rest, matched) = bracket_right(input).expect("should parse ']' symbol");
            assert_eq!(matched.lexeme_str, "]");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn caret_symbol() {
            let input = InputSpan::new_extra("^ rest", Config::default());
            let (rest, matched) = caret(input).expect("should parse '^' symbol");
            assert_eq!(matched.lexeme_str, "^");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn colon_symbol() {
            let input = InputSpan::new_extra(": rest", Config::default());
            let (rest, matched) = colon(input).expect("should parse ':' symbol");
            assert_eq!(matched.lexeme_str, ":");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn comma_symbol() {
            let input = InputSpan::new_extra(", rest", Config::default());
            let (rest, matched) = comma(input).expect("should parse ',' symbol");
            assert_eq!(matched.lexeme_str, ",");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn dollar_symbol() {
            let input = InputSpan::new_extra("$ rest", Config::default());
            let (rest, matched) = dollar(input).expect("should parse '$' symbol");
            assert_eq!(matched.lexeme_str, "$");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn dot_symbol() {
            let input = InputSpan::new_extra(". rest", Config::default());
            let (rest, matched) = dot(input).expect("should parse '.' symbol");
            assert_eq!(matched.lexeme_str, ".");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn dot_dot_symbol() {
            let input = InputSpan::new_extra(".. rest", Config::default());
            let (rest, matched) = dot_dot(input).expect("should parse '..' symbol");
            assert_eq!(matched.lexeme_str, "..");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn paren_left_symbol() {
            let input = InputSpan::new_extra("( rest", Config::default());
            let (rest, matched) = paren_left(input).expect("should parse '(' symbol");
            assert_eq!(matched.lexeme_str, "(");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn paren_right_symbol() {
            let input = InputSpan::new_extra(") rest", Config::default());
            let (rest, matched) = paren_right(input).expect("should parse ')' symbol");
            assert_eq!(matched.lexeme_str, ")");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn percent_symbol() {
            let input = InputSpan::new_extra("% rest", Config::default());
            let (rest, matched) = percent(input).expect("should parse '%' symbol");
            assert_eq!(matched.lexeme_str, "%");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn plus_symbol() {
            let input = InputSpan::new_extra("+ rest", Config::default());
            let (rest, matched) = plus(input).expect("should parse '+' symbol");
            assert_eq!(matched.lexeme_str, "+");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn bang_equals_symbol() {
            let input = InputSpan::new_extra("!= rest", Config::default());
            let (rest, matched) = bang_equals(input).expect("should parse '!=' symbol");
            assert_eq!(matched.lexeme_str, "!=");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn equals_equals_symbol() {
            let input = InputSpan::new_extra("== rest", Config::default());
            let (rest, matched) = equals_equals(input).expect("should parse '==' symbol");
            assert_eq!(matched.lexeme_str, "==");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn greater_than_equals_symbol() {
            let input = InputSpan::new_extra(">= rest", Config::default());
            let (rest, matched) = greater_than_equals(input).expect("should parse '>=' symbol");
            assert_eq!(matched.lexeme_str, ">=");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn less_than_equals_symbol() {
            let input = InputSpan::new_extra("<= rest", Config::default());
            let (rest, matched) = less_than_equals(input).expect("should parse '<=' symbol");
            assert_eq!(matched.lexeme_str, "<=");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn star_star_symbol() {
            let input = InputSpan::new_extra("** rest", Config::default());
            let (rest, matched) = star_star(input).expect("should parse '**' symbol");
            assert_eq!(matched.lexeme_str, "**");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn equals_symbol() {
            let input = InputSpan::new_extra("= rest", Config::default());
            let (rest, matched) = equals(input).expect("should parse '=' symbol");
            assert_eq!(matched.lexeme_str, "=");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn greater_than_symbol() {
            let input = InputSpan::new_extra("> rest", Config::default());
            let (rest, matched) = greater_than(input).expect("should parse '>' symbol");
            assert_eq!(matched.lexeme_str, ">");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn less_than_symbol() {
            let input = InputSpan::new_extra("< rest", Config::default());
            let (rest, matched) = less_than(input).expect("should parse '<' symbol");
            assert_eq!(matched.lexeme_str, "<");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn minus_symbol() {
            let input = InputSpan::new_extra("- rest", Config::default());
            let (rest, matched) = minus(input).expect("should parse '-' symbol");
            assert_eq!(matched.lexeme_str, "-");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn star_symbol() {
            let input = InputSpan::new_extra("* rest", Config::default());
            let (rest, matched) = star(input).expect("should parse '*' symbol");
            assert_eq!(matched.lexeme_str, "*");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn slash_symbol() {
            let input = InputSpan::new_extra("/ rest", Config::default());
            let (rest, matched) = slash(input).expect("should parse '/' symbol");
            assert_eq!(matched.lexeme_str, "/");
            assert_eq!(rest.fragment(), &"rest");
        }

        // Edge cases for all symbol types
        #[test]
        fn symbol_at_end_of_file() {
            let input = InputSpan::new_extra("+", Config::default());
            let (rest, matched) = plus(input).expect("should parse '+' at end of file");
            assert_eq!(matched.lexeme_str, "+");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn multi_char_symbol_at_end_of_file() {
            let input = InputSpan::new_extra("!=", Config::default());
            let (rest, matched) = bang_equals(input).expect("should parse '!=' at end of file");
            assert_eq!(matched.lexeme_str, "!=");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn symbol_with_trailing_whitespace() {
            let input = InputSpan::new_extra("+   rest", Config::default());
            let (rest, matched) = plus(input).expect("should parse '+' with trailing whitespace");
            assert_eq!(matched.lexeme_str, "+");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn multi_char_symbol_with_trailing_whitespace() {
            let input = InputSpan::new_extra("!=   rest", Config::default());
            let (rest, matched) =
                bang_equals(input).expect("should parse '!=' with trailing whitespace");
            assert_eq!(matched.lexeme_str, "!=");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn symbol_with_tab() {
            let input = InputSpan::new_extra("+\trest", Config::default());
            let (rest, matched) = plus(input).expect("should parse '+' with tab");
            assert_eq!(matched.lexeme_str, "+");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn multi_char_symbol_with_tab() {
            let input = InputSpan::new_extra("!=\trest", Config::default());
            let (rest, matched) = bang_equals(input).expect("should parse '!=' with tab");
            assert_eq!(matched.lexeme_str, "!=");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn symbol_with_newline() {
            let input = InputSpan::new_extra("+\nrest", Config::default());
            let (rest, matched) = plus(input).expect("should parse '+' with newline");
            assert_eq!(matched.lexeme_str, "+");
            assert_eq!(rest.fragment(), &"\nrest");
        }

        #[test]
        fn multi_char_symbol_with_newline() {
            let input = InputSpan::new_extra("!=\nrest", Config::default());
            let (rest, matched) = bang_equals(input).expect("should parse '!=' with newline");
            assert_eq!(matched.lexeme_str, "!=");
            assert_eq!(rest.fragment(), &"\nrest");
        }

        #[test]
        fn symbol_with_carriage_return() {
            let input = InputSpan::new_extra("+\rrest", Config::default());
            let (rest, matched) = plus(input).expect("should parse '+' with carriage return");
            assert_eq!(matched.lexeme_str, "+");
            assert_eq!(rest.fragment(), &"\rrest");
        }
    }

    mod error {
        use crate::token::error::ExpectKind;

        use super::*;

        #[test]
        fn empty_input() {
            let input = InputSpan::new_extra("", Config::default());
            let res = plus(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(_), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::Symbol(ExpectSymbol::Plus))
            ));
        }

        #[test]
        fn symbol_not_at_start() {
            let input = InputSpan::new_extra("foo + bar", Config::default());
            let res = plus(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(_), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::Symbol(ExpectSymbol::Plus))
            ));
        }

        #[test]
        fn whitespace_only() {
            let input = InputSpan::new_extra("   ", Config::default());
            let res = plus(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(_), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::Symbol(ExpectSymbol::Plus))
            ));
        }

        #[test]
        fn wrong_symbol() {
            let input = InputSpan::new_extra("x", Config::default());
            let res = plus(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(_), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::Symbol(ExpectSymbol::Plus))
            ));
        }

        #[test]
        fn dot_not_dot_dot() {
            let input = InputSpan::new_extra(".. rest", Config::default());
            let res = dot(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(_), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::Symbol(ExpectSymbol::Dot))
            ));
        }

        #[test]
        fn equals_not_equals_equals() {
            let input = InputSpan::new_extra("== rest", Config::default());
            let res = equals(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(_), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::Symbol(ExpectSymbol::Equals))
            ));
        }

        #[test]
        fn greater_than_not_greater_than_equals() {
            let input = InputSpan::new_extra(">= rest", Config::default());
            let res = greater_than(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(_), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::Symbol(ExpectSymbol::GreaterThan))
            ));
        }

        #[test]
        fn less_than_not_less_than_equals() {
            let input = InputSpan::new_extra("<= rest", Config::default());
            let res = less_than(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(_), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::Symbol(ExpectSymbol::LessThan))
            ));
        }

        #[test]
        fn star_not_star_star() {
            let input = InputSpan::new_extra("** rest", Config::default());
            let res = star(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(_), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::Symbol(ExpectSymbol::Star))
            ));
        }

        #[test]
        fn multi_char_symbol_partial() {
            let input = InputSpan::new_extra("!", Config::default());
            let res = bang_equals(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(_), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::Symbol(ExpectSymbol::BangEquals))
            ));
        }

        #[test]
        fn multi_char_symbol_wrong_second_char() {
            let input = InputSpan::new_extra("!x rest", Config::default());
            let res = bang_equals(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(_), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::Symbol(ExpectSymbol::BangEquals))
            ));
        }
    }

    mod next_char_is_not {
        use super::*;

        #[test]
        fn next_char_is_not_succeeds() {
            let input = InputSpan::new_extra("abc", Config::default());
            let (rest, ()) = next_char_is_not('b')
                .parse(input)
                .expect("next char should not be 'b'");
            assert_eq!(rest.fragment(), &"abc");
        }

        #[test]
        fn next_char_is_not_succeeds_with_eof() {
            let input = InputSpan::new_extra("", Config::default());
            let (rest, ()) = next_char_is_not('b')
                .parse(input)
                .expect("next char should not be 'b' at EOF");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn next_char_is_not_fails() {
            let input = InputSpan::new_extra("abc", Config::default());
            let res = next_char_is_not('a').parse(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::NomError(_), got {res:?}");
            };

            assert!(matches!(token_error.kind, TokenErrorKind::NomError(_)));
        }

        #[test]
        fn next_char_is_not_with_whitespace() {
            let input = InputSpan::new_extra(" b", Config::default());
            let (rest, ()) = next_char_is_not('a')
                .parse(input)
                .expect("next char should not be 'a'");
            assert_eq!(rest.fragment(), &" b");
        }

        #[test]
        fn next_char_is_not_with_special_characters() {
            let input = InputSpan::new_extra("!bc", Config::default());
            let (rest, ()) = next_char_is_not('a')
                .parse(input)
                .expect("next char should not be 'a'");
            assert_eq!(rest.fragment(), &"!bc");
        }

        #[test]
        fn next_char_is_not_with_numbers() {
            let input = InputSpan::new_extra("123", Config::default());
            let (rest, ()) = next_char_is_not('a')
                .parse(input)
                .expect("next char should not be 'a'");
            assert_eq!(rest.fragment(), &"123");
        }

        #[test]
        fn next_char_is_not_with_unicode() {
            let input = InputSpan::new_extra("αβγ", Config::default());
            let (rest, ()) = next_char_is_not('a')
                .parse(input)
                .expect("next char should not be 'a'");
            assert_eq!(rest.fragment(), &"αβγ");
        }

        #[test]
        fn next_char_is_not_with_emoji() {
            let input = InputSpan::new_extra("🚀bc", Config::default());
            let (rest, ()) = next_char_is_not('a')
                .parse(input)
                .expect("next char should not be 'a'");
            assert_eq!(rest.fragment(), &"🚀bc");
        }
    }
}
