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
    Parser, Result, Span,
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

    move |input: Span<'a>| parser.parse(input)
}

/// Parses the '!=' symbol token.
pub fn bang_equals(input: Span) -> Result<Token, TokenError> {
    token(
        tag("!="),
        TokenError::expected_symbol(ExpectSymbol::BangEquals),
    )
    .parse(input)
}

/// Parses the '|' symbol token.
pub fn bar(input: Span) -> Result<Token, TokenError> {
    token(char('|'), TokenError::expected_symbol(ExpectSymbol::Bar)).parse(input)
}

/// Parses the '{' symbol token.
pub fn brace_left(input: Span) -> Result<Token, TokenError> {
    token(
        char('{'),
        TokenError::expected_symbol(ExpectSymbol::BraceLeft),
    )
    .parse(input)
}

/// Parses the '}' symbol token.
pub fn brace_right(input: Span) -> Result<Token, TokenError> {
    token(
        char('}'),
        TokenError::expected_symbol(ExpectSymbol::BraceRight),
    )
    .parse(input)
}

/// Parses the '[' symbol token.
pub fn bracket_left(input: Span) -> Result<Token, TokenError> {
    token(
        char('['),
        TokenError::expected_symbol(ExpectSymbol::BracketLeft),
    )
    .parse(input)
}

/// Parses the ']' symbol token.
pub fn bracket_right(input: Span) -> Result<Token, TokenError> {
    token(
        char(']'),
        TokenError::expected_symbol(ExpectSymbol::BracketRight),
    )
    .parse(input)
}

/// Parses the '^' symbol token.
pub fn caret(input: Span) -> Result<Token, TokenError> {
    token(char('^'), TokenError::expected_symbol(ExpectSymbol::Caret)).parse(input)
}

/// Parses the ':' symbol token.
pub fn colon(input: Span) -> Result<Token, TokenError> {
    token(char(':'), TokenError::expected_symbol(ExpectSymbol::Colon)).parse(input)
}

/// Parses the ',' symbol token.
pub fn comma(input: Span) -> Result<Token, TokenError> {
    token(char(','), TokenError::expected_symbol(ExpectSymbol::Comma)).parse(input)
}

/// Parses the '$' symbol token.
pub fn dollar(input: Span) -> Result<Token, TokenError> {
    token(char('$'), TokenError::expected_symbol(ExpectSymbol::Dollar)).parse(input)
}

/// Parses the '.' symbol token.
pub fn dot(input: Span) -> Result<Token, TokenError> {
    token(char('.'), TokenError::expected_symbol(ExpectSymbol::Dot)).parse(input)
}

/// Parses the '=' symbol token.
pub fn equals(input: Span) -> Result<Token, TokenError> {
    token(
        char('=').and(next_char_is_not('=')),
        TokenError::expected_symbol(ExpectSymbol::Equals),
    )
    .parse(input)
}

/// Parses the '==' symbol token.
pub fn equals_equals(input: Span) -> Result<Token, TokenError> {
    token(
        tag("=="),
        TokenError::expected_symbol(ExpectSymbol::EqualsEquals),
    )
    .parse(input)
}

/// Parses the '>' symbol token.
pub fn greater_than(input: Span) -> Result<Token, TokenError> {
    token(
        char('>').and(next_char_is_not('=')),
        TokenError::expected_symbol(ExpectSymbol::GreaterThan),
    )
    .parse(input)
}

/// Parses the '>=' symbol token.
pub fn greater_than_equals(input: Span) -> Result<Token, TokenError> {
    token(
        tag(">="),
        TokenError::expected_symbol(ExpectSymbol::GreaterThanEquals),
    )
    .parse(input)
}

/// Parses the '<' symbol token.
pub fn less_than(input: Span) -> Result<Token, TokenError> {
    token(
        char('<').and(next_char_is_not('=')),
        TokenError::expected_symbol(ExpectSymbol::LessThan),
    )
    .parse(input)
}

/// Parses the '<=' symbol token.
pub fn less_than_equals(input: Span) -> Result<Token, TokenError> {
    token(
        tag("<="),
        TokenError::expected_symbol(ExpectSymbol::LessThanEquals),
    )
    .parse(input)
}

/// Parses the '-' symbol token.
pub fn minus(input: Span) -> Result<Token, TokenError> {
    token(
        char('-').and(next_char_is_not('-')),
        TokenError::expected_symbol(ExpectSymbol::Minus),
    )
    .parse(input)
}

/// Parses the '--' symbol token.
pub fn minus_minus(input: Span) -> Result<Token, TokenError> {
    token(
        tag("--"),
        TokenError::expected_symbol(ExpectSymbol::MinusMinus),
    )
    .parse(input)
}

/// Parses the '(' symbol token.
pub fn paren_left(input: Span) -> Result<Token, TokenError> {
    token(
        char('('),
        TokenError::expected_symbol(ExpectSymbol::ParenLeft),
    )
    .parse(input)
}

/// Parses the ')' symbol token.
pub fn paren_right(input: Span) -> Result<Token, TokenError> {
    token(
        char(')'),
        TokenError::expected_symbol(ExpectSymbol::ParenRight),
    )
    .parse(input)
}

/// Parses the '%' symbol token.
pub fn percent(input: Span) -> Result<Token, TokenError> {
    token(
        char('%'),
        TokenError::expected_symbol(ExpectSymbol::Percent),
    )
    .parse(input)
}

/// Parses the '+' symbol token.
pub fn plus(input: Span) -> Result<Token, TokenError> {
    token(char('+'), TokenError::expected_symbol(ExpectSymbol::Plus)).parse(input)
}

/// Parses the '*' symbol token.
pub fn star(input: Span) -> Result<Token, TokenError> {
    token(
        char('*').and(next_char_is_not('*')),
        TokenError::expected_symbol(ExpectSymbol::Star),
    )
    .parse(input)
}

/// Parses the '**' symbol token.
pub fn star_star(input: Span) -> Result<Token, TokenError> {
    token(
        tag("**"),
        TokenError::expected_symbol(ExpectSymbol::StarStar),
    )
    .parse(input)
}

/// Parses the '/' symbol token.
pub fn slash(input: Span) -> Result<Token, TokenError> {
    token(
        char('/').and(next_char_is_not('/')),
        TokenError::expected_symbol(ExpectSymbol::Slash),
    )
    .parse(input)
}

/// Parses the '//' symbol token.
pub fn slash_slash(input: Span) -> Result<Token, TokenError> {
    token(
        tag("//"),
        TokenError::expected_symbol(ExpectSymbol::SlashSlash),
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Config, Span};

    #[test]
    fn test_bang_equals() {
        let input = Span::new_extra("!= rest", Config::default());
        let (rest, matched) = bang_equals(input).expect("should parse '!=' symbol");
        assert_eq!(matched.lexeme(), "!=");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_bar() {
        let input = Span::new_extra("| rest", Config::default());
        let (rest, matched) = bar(input).expect("should parse '|' symbol");
        assert_eq!(matched.lexeme(), "|");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_brace_left() {
        let input = Span::new_extra("{ rest", Config::default());
        let (rest, matched) = brace_left(input).expect("should parse '{' symbol");
        assert_eq!(matched.lexeme(), "{");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_brace_right() {
        let input = Span::new_extra("} rest", Config::default());
        let (rest, matched) = brace_right(input).expect("should parse '}' symbol");
        assert_eq!(matched.lexeme(), "}");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_bracket_left() {
        let input = Span::new_extra("[ rest", Config::default());
        let (rest, matched) = bracket_left(input).expect("should parse '[' symbol");
        assert_eq!(matched.lexeme(), "[");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_bracket_right() {
        let input = Span::new_extra("] rest", Config::default());
        let (rest, matched) = bracket_right(input).expect("should parse ']' symbol");
        assert_eq!(matched.lexeme(), "]");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_caret() {
        let input = Span::new_extra("^ rest", Config::default());
        let (rest, matched) = caret(input).expect("should parse '^' symbol");
        assert_eq!(matched.lexeme(), "^");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_colon() {
        let input = Span::new_extra(": rest", Config::default());
        let (rest, matched) = colon(input).expect("should parse ':' symbol");
        assert_eq!(matched.lexeme(), ":");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_comma() {
        let input = Span::new_extra(", rest", Config::default());
        let (rest, matched) = comma(input).expect("should parse ',' symbol");
        assert_eq!(matched.lexeme(), ",");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_dollar() {
        let input = Span::new_extra("$ rest", Config::default());
        let (rest, matched) = dollar(input).expect("should parse '$' symbol");
        assert_eq!(matched.lexeme(), "$");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_dot() {
        let input = Span::new_extra(". rest", Config::default());
        let (rest, matched) = dot(input).expect("should parse '.' symbol");
        assert_eq!(matched.lexeme(), ".");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_equals() {
        let input = Span::new_extra("= rest", Config::default());
        let (rest, matched) = equals(input).expect("should parse '=' symbol");
        assert_eq!(matched.lexeme(), "=");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_equals_not_equals_equals() {
        let input = Span::new_extra("== rest", Config::default());
        let res = equals(input);
        assert!(res.is_err(), "should not parse '==' as '='");
    }

    #[test]
    fn test_equals_equals() {
        let input = Span::new_extra("== rest", Config::default());
        let (rest, matched) = equals_equals(input).expect("should parse '==' symbol");
        assert_eq!(matched.lexeme(), "==");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_greater_than() {
        let input = Span::new_extra("> rest", Config::default());
        let (rest, matched) = greater_than(input).expect("should parse '>' symbol");
        assert_eq!(matched.lexeme(), ">");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_greater_than_not_greater_than_equals() {
        let input = Span::new_extra(">= rest", Config::default());
        let res = greater_than(input);
        assert!(res.is_err(), "should not parse '>=' as '>'");
    }

    #[test]
    fn test_greater_than_equals() {
        let input = Span::new_extra(">= rest", Config::default());
        let (rest, matched) = greater_than_equals(input).expect("should parse '>=' symbol");
        assert_eq!(matched.lexeme(), ">=");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_less_than() {
        let input = Span::new_extra("< rest", Config::default());
        let (rest, matched) = less_than(input).expect("should parse '<' symbol");
        assert_eq!(matched.lexeme(), "<");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_less_than_not_less_than_equals() {
        let input = Span::new_extra("<= rest", Config::default());
        let res = less_than(input);
        assert!(res.is_err(), "should not parse '<=' as '<'");
    }

    #[test]
    fn test_less_than_equals() {
        let input = Span::new_extra("<= rest", Config::default());
        let (rest, matched) = less_than_equals(input).expect("should parse '<=' symbol");
        assert_eq!(matched.lexeme(), "<=");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_minus() {
        let input = Span::new_extra("- rest", Config::default());
        let (rest, matched) = minus(input).expect("should parse '-' symbol");
        assert_eq!(matched.lexeme(), "-");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_minus_not_minus_minus() {
        let input = Span::new_extra("-- rest", Config::default());
        let res = minus(input);
        assert!(res.is_err(), "should not parse '--' as '-'");
    }

    #[test]
    fn test_minus_minus() {
        let input = Span::new_extra("-- rest", Config::default());
        let (rest, matched) = minus_minus(input).expect("should parse '--' symbol");
        assert_eq!(matched.lexeme(), "--");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_paren_left() {
        let input = Span::new_extra("( rest", Config::default());
        let (rest, matched) = paren_left(input).expect("should parse '(' symbol");
        assert_eq!(matched.lexeme(), "(");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_paren_right() {
        let input = Span::new_extra(") rest", Config::default());
        let (rest, matched) = paren_right(input).expect("should parse ')' symbol");
        assert_eq!(matched.lexeme(), ")");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_percent() {
        let input = Span::new_extra("% rest", Config::default());
        let (rest, matched) = percent(input).expect("should parse '%' symbol");
        assert_eq!(matched.lexeme(), "%");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_plus() {
        let input = Span::new_extra("+ rest", Config::default());
        let (rest, matched) = plus(input).expect("should parse '+' symbol");
        assert_eq!(matched.lexeme(), "+");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_star() {
        let input = Span::new_extra("* rest", Config::default());
        let (rest, matched) = star(input).expect("should parse '*' symbol");
        assert_eq!(matched.lexeme(), "*");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_star_not_star_star() {
        let input = Span::new_extra("** rest", Config::default());
        let res = star(input);
        assert!(res.is_err(), "should not parse '**' as '*'");
    }

    #[test]
    fn test_star_star() {
        let input = Span::new_extra("** rest", Config::default());
        let (rest, matched) = star_star(input).expect("should parse '**' symbol");
        assert_eq!(matched.lexeme(), "**");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_slash() {
        let input = Span::new_extra("/ rest", Config::default());
        let (rest, matched) = slash(input).expect("should parse '/' symbol");
        assert_eq!(matched.lexeme(), "/");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_slash_not_slash_slash() {
        let input = Span::new_extra("// rest", Config::default());
        let res = slash(input);
        assert!(res.is_err(), "should not parse '//' as '/'");
    }

    #[test]
    fn test_slash_slash() {
        let input = Span::new_extra("// rest", Config::default());
        let (rest, matched) = slash_slash(input).expect("should parse '//' symbol");
        assert_eq!(matched.lexeme(), "//");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_symbol_with_trailing_whitespace() {
        let input = Span::new_extra("+   rest", Config::default());
        let (rest, matched) = plus(input).expect("should parse '+' with trailing whitespace");
        assert_eq!(matched.lexeme(), "+");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_symbol_not_at_start() {
        let input = Span::new_extra("foo + bar", Config::default());
        let res = plus(input);
        assert!(res.is_err(), "should not parse '+' if not at start");
    }

    #[test]
    fn test_next_char_is_not_succeeds() {
        let input = Span::new_extra("abc", Config::default());
        let (rest, _) = next_char_is_not('b')
            .parse(input)
            .expect("next char should not be 'b'");
        assert_eq!(rest.fragment(), &"abc");
    }

    #[test]
    fn test_next_char_is_not_succeeds_with_eof() {
        let input = Span::new_extra("", Config::default());
        let (rest, _) = next_char_is_not('b')
            .parse(input)
            .expect("next char should not be 'b'");
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_next_char_is_not_fails() {
        let input = Span::new_extra("abc", Config::default());
        let res = next_char_is_not('a').parse(input);
        assert!(res.is_err(), "should not succeed if next char is 'a'");
    }
}
