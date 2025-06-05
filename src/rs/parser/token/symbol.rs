//! Provides parsers for symbols in the Oneil language.
//!
//! This module contains parsers for all symbol tokens in the Oneil language,
//! including operators, delimiters, and other special characters.
//!
//! # Examples
//!
//! ```
//! use oneil::parser::token::symbol::{bang_equals, brace_left, brace_right};
//! use oneil::parser::Span;
//!
//! // Parse a not-equals operator
//! let input = Span::new("!= rest");
//! let (rest, matched) = bang_equals(input).unwrap();
//! assert_eq!(matched.fragment(), &"!=");
//!
//! // Parse braces
//! let input = Span::new("{ rest");
//! let (rest, matched) = brace_left(input).unwrap();
//! assert_eq!(matched.fragment(), &"{");
//!
//! let input = Span::new("} rest");
//! let (rest, matched) = brace_right(input).unwrap();
//! assert_eq!(matched.fragment(), &"}");
//! ```

use nom::{
    Parser as _,
    bytes::complete::tag,
    character::complete::{char, satisfy},
    combinator::{eof, peek, value},
};

use super::{Result, Span, util::token};

fn next_char_is_not(c: char) -> impl Fn(Span) -> Result<()> {
    move |input: Span| {
        let next_char_is_not_c = peek(satisfy(|next_char: char| next_char != c)).map(|_| ());
        let reached_end_of_file = eof.map(|_| ());
        value((), next_char_is_not_c.or(reached_end_of_file)).parse(input)
    }
}

/// Parses the '!=' symbol token.
pub fn bang_equals(input: Span) -> Result<Span> {
    token(tag("!=")).parse(input)
}

/// Parses the '|' symbol token.
pub fn bar(input: Span) -> Result<Span> {
    token(char('|')).parse(input)
}

/// Parses the '{' symbol token.
pub fn brace_left(input: Span) -> Result<Span> {
    token(char('{')).parse(input)
}

/// Parses the '}' symbol token.
pub fn brace_right(input: Span) -> Result<Span> {
    token(char('}')).parse(input)
}

/// Parses the '[' symbol token.
pub fn bracket_left(input: Span) -> Result<Span> {
    token(char('[')).parse(input)
}

/// Parses the ']' symbol token.
pub fn bracket_right(input: Span) -> Result<Span> {
    token(char(']')).parse(input)
}

/// Parses the '^' symbol token.
pub fn caret(input: Span) -> Result<Span> {
    token(char('^')).parse(input)
}

/// Parses the ':' symbol token.
pub fn colon(input: Span) -> Result<Span> {
    token(char(':')).parse(input)
}

/// Parses the ',' symbol token.
pub fn comma(input: Span) -> Result<Span> {
    token(char(',')).parse(input)
}

/// Parses the '$' symbol token.
pub fn dollar(input: Span) -> Result<Span> {
    token(char('$')).parse(input)
}

/// Parses the '.' symbol token.
pub fn dot(input: Span) -> Result<Span> {
    token(char('.')).parse(input)
}

/// Parses the '=' symbol token.
pub fn equals(input: Span) -> Result<Span> {
    token(char('=').and(next_char_is_not('='))).parse(input)
}

/// Parses the '==' symbol token.
pub fn equals_equals(input: Span) -> Result<Span> {
    token(tag("==")).parse(input)
}

/// Parses the '>' symbol token.
pub fn greater_than(input: Span) -> Result<Span> {
    token(char('>').and(next_char_is_not('='))).parse(input)
}

/// Parses the '>=' symbol token.
pub fn greater_than_equals(input: Span) -> Result<Span> {
    token(tag(">=")).parse(input)
}

/// Parses the '<' symbol token.
pub fn less_than(input: Span) -> Result<Span> {
    token(char('<').and(next_char_is_not('='))).parse(input)
}

/// Parses the '<=' symbol token.
pub fn less_than_equals(input: Span) -> Result<Span> {
    token(tag("<=")).parse(input)
}

/// Parses the '-' symbol token.
pub fn minus(input: Span) -> Result<Span> {
    token(char('-').and(next_char_is_not('-'))).parse(input)
}

/// Parses the '--' symbol token.
pub fn minus_minus(input: Span) -> Result<Span> {
    token(tag("--")).parse(input)
}

/// Parses the '(' symbol token.
pub fn paren_left(input: Span) -> Result<Span> {
    token(char('(')).parse(input)
}

/// Parses the ')' symbol token.
pub fn paren_right(input: Span) -> Result<Span> {
    token(char(')')).parse(input)
}

/// Parses the '%' symbol token.
pub fn percent(input: Span) -> Result<Span> {
    token(char('%')).parse(input)
}

/// Parses the '+' symbol token.
pub fn plus(input: Span) -> Result<Span> {
    token(char('+')).parse(input)
}

/// Parses the '*' symbol token.
pub fn star(input: Span) -> Result<Span> {
    token(char('*').and(next_char_is_not('*'))).parse(input)
}

/// Parses the '**' symbol token.
pub fn star_star(input: Span) -> Result<Span> {
    token(tag("**")).parse(input)
}

/// Parses the '/' symbol token.
pub fn slash(input: Span) -> Result<Span> {
    token(char('/').and(next_char_is_not('/'))).parse(input)
}

/// Parses the '//' symbol token.
pub fn slash_slash(input: Span) -> Result<Span> {
    token(tag("//")).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::token::Span;

    #[test]
    fn test_bang_equals() {
        let input = Span::new("!= rest");
        let (rest, matched) = bang_equals(input).expect("should parse '!=' symbol");
        assert_eq!(matched.fragment(), &"!=");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_bar() {
        let input = Span::new("| rest");
        let (rest, matched) = bar(input).expect("should parse '|' symbol");
        assert_eq!(matched.fragment(), &"|");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_brace_left() {
        let input = Span::new("{ rest");
        let (rest, matched) = brace_left(input).expect("should parse '{' symbol");
        assert_eq!(matched.fragment(), &"{");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_brace_right() {
        let input = Span::new("} rest");
        let (rest, matched) = brace_right(input).expect("should parse '}' symbol");
        assert_eq!(matched.fragment(), &"}");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_bracket_left() {
        let input = Span::new("[ rest");
        let (rest, matched) = bracket_left(input).expect("should parse '[' symbol");
        assert_eq!(matched.fragment(), &"[");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_bracket_right() {
        let input = Span::new("] rest");
        let (rest, matched) = bracket_right(input).expect("should parse ']' symbol");
        assert_eq!(matched.fragment(), &"]");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_caret() {
        let input = Span::new("^ rest");
        let (rest, matched) = caret(input).expect("should parse '^' symbol");
        assert_eq!(matched.fragment(), &"^");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_colon() {
        let input = Span::new(": rest");
        let (rest, matched) = colon(input).expect("should parse ':' symbol");
        assert_eq!(matched.fragment(), &":");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_comma() {
        let input = Span::new(", rest");
        let (rest, matched) = comma(input).expect("should parse ',' symbol");
        assert_eq!(matched.fragment(), &",");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_dollar() {
        let input = Span::new("$ rest");
        let (rest, matched) = dollar(input).expect("should parse '$' symbol");
        assert_eq!(matched.fragment(), &"$");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_dot() {
        let input = Span::new(". rest");
        let (rest, matched) = dot(input).expect("should parse '.' symbol");
        assert_eq!(matched.fragment(), &".");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_equals() {
        let input = Span::new("= rest");
        let (rest, matched) = equals(input).expect("should parse '=' symbol");
        assert_eq!(matched.fragment(), &"=");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_equals_not_equals_equals() {
        let input = Span::new("== rest");
        let res = equals(input);
        assert!(res.is_err(), "should not parse '==' as '='");
    }

    #[test]
    fn test_equals_equals() {
        let input = Span::new("== rest");
        let (rest, matched) = equals_equals(input).expect("should parse '==' symbol");
        assert_eq!(matched.fragment(), &"==");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_greater_than() {
        let input = Span::new("> rest");
        let (rest, matched) = greater_than(input).expect("should parse '>' symbol");
        assert_eq!(matched.fragment(), &">");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_greater_than_not_greater_than_equals() {
        let input = Span::new(">= rest");
        let res = greater_than(input);
        assert!(res.is_err(), "should not parse '>=' as '>'");
    }

    #[test]
    fn test_greater_than_equals() {
        let input = Span::new(">= rest");
        let (rest, matched) = greater_than_equals(input).expect("should parse '>=' symbol");
        assert_eq!(matched.fragment(), &">=");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_less_than() {
        let input = Span::new("< rest");
        let (rest, matched) = less_than(input).expect("should parse '<' symbol");
        assert_eq!(matched.fragment(), &"<");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_less_than_not_less_than_equals() {
        let input = Span::new("<= rest");
        let res = less_than(input);
        assert!(res.is_err(), "should not parse '<=' as '<'");
    }

    #[test]
    fn test_less_than_equals() {
        let input = Span::new("<= rest");
        let (rest, matched) = less_than_equals(input).expect("should parse '<=' symbol");
        assert_eq!(matched.fragment(), &"<=");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_minus() {
        let input = Span::new("- rest");
        let (rest, matched) = minus(input).expect("should parse '-' symbol");
        assert_eq!(matched.fragment(), &"-");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_minus_not_minus_minus() {
        let input = Span::new("-- rest");
        let res = minus(input);
        assert!(res.is_err(), "should not parse '--' as '-'");
    }

    #[test]
    fn test_minus_minus() {
        let input = Span::new("-- rest");
        let (rest, matched) = minus_minus(input).expect("should parse '--' symbol");
        assert_eq!(matched.fragment(), &"--");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_paren_left() {
        let input = Span::new("( rest");
        let (rest, matched) = paren_left(input).expect("should parse '(' symbol");
        assert_eq!(matched.fragment(), &"(");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_paren_right() {
        let input = Span::new(") rest");
        let (rest, matched) = paren_right(input).expect("should parse ')' symbol");
        assert_eq!(matched.fragment(), &")");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_percent() {
        let input = Span::new("% rest");
        let (rest, matched) = percent(input).expect("should parse '%' symbol");
        assert_eq!(matched.fragment(), &"%");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_plus() {
        let input = Span::new("+ rest");
        let (rest, matched) = plus(input).expect("should parse '+' symbol");
        assert_eq!(matched.fragment(), &"+");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_star() {
        let input = Span::new("* rest");
        let (rest, matched) = star(input).expect("should parse '*' symbol");
        assert_eq!(matched.fragment(), &"*");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_star_not_star_star() {
        let input = Span::new("** rest");
        let res = star(input);
        assert!(res.is_err(), "should not parse '**' as '*'");
    }

    #[test]
    fn test_star_star() {
        let input = Span::new("** rest");
        let (rest, matched) = star_star(input).expect("should parse '**' symbol");
        assert_eq!(matched.fragment(), &"**");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_slash() {
        let input = Span::new("/ rest");
        let (rest, matched) = slash(input).expect("should parse '/' symbol");
        assert_eq!(matched.fragment(), &"/");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_slash_not_slash_slash() {
        let input = Span::new("// rest");
        let res = slash(input);
        assert!(res.is_err(), "should not parse '//' as '/'");
    }

    #[test]
    fn test_slash_slash() {
        let input = Span::new("// rest");
        let (rest, matched) = slash_slash(input).expect("should parse '//' symbol");
        assert_eq!(matched.fragment(), &"//");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_symbol_with_trailing_whitespace() {
        let input = Span::new("+   rest");
        let (rest, matched) = plus(input).expect("should parse '+' with trailing whitespace");
        assert_eq!(matched.fragment(), &"+");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_symbol_not_at_start() {
        let input = Span::new("foo + bar");
        let res = plus(input);
        assert!(res.is_err(), "should not parse '+' if not at start");
    }

    #[test]
    fn test_next_char_is_not_succeeds() {
        let input = Span::new("abc");
        let (rest, _) = next_char_is_not('b')(input).expect("next char should not be 'b'");
        assert_eq!(rest.fragment(), &"abc");
    }

    #[test]
    fn test_next_char_is_not_succeeds_with_eof() {
        let input = Span::new("");
        let (rest, _) = next_char_is_not('b')(input).expect("next char should not be 'b'");
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_next_char_is_not_fails() {
        let input = Span::new("abc");
        let res = next_char_is_not('a')(input);
        assert!(res.is_err(), "should not succeed if next char is 'a'");
    }
}
