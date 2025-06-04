use nom::{
    Parser as _,
    character::complete::space0,
    combinator::{recognize, value},
    sequence::terminated,
};

use super::{Parser, Result, Span};

pub fn inline_whitespace(input: Span) -> Result<()> {
    value((), space0).parse(input)
}

pub fn token<'a, F, O>(f: F) -> impl Parser<'a, Span<'a>>
where
    F: Parser<'a, O>,
{
    terminated(recognize(f), inline_whitespace)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_whitespace_spaces() {
        let input = Span::new("   abc");
        let (rest, _) = inline_whitespace(input).expect("should parse leading spaces");
        assert_eq!(rest.fragment(), &"abc");
    }

    #[test]
    fn test_inline_whitespace_tabs() {
        let input = Span::new("\t\tfoo");
        let (rest, _) = inline_whitespace(input).expect("should parse leading tabs");
        assert_eq!(rest.fragment(), &"foo");
    }

    #[test]
    fn test_inline_whitespace_none() {
        let input = Span::new("bar");
        let (rest, _) = inline_whitespace(input).expect("should parse no whitespace");
        assert_eq!(rest.fragment(), &"bar");
    }

    #[test]
    fn test_token_with_whitespace() {
        use nom::bytes::complete::tag;
        let mut parser = token(tag("foo"));
        let input = Span::new("foo   bar");
        let (rest, matched) = parser
            .parse(input)
            .expect("should parse token with trailing whitespace");
        assert_eq!(matched.fragment(), &"foo");
        assert_eq!(rest.fragment(), &"bar");
    }

    #[test]
    fn test_token_no_match() {
        use nom::bytes::complete::tag;
        let mut parser = token(tag("baz"));
        let input = Span::new("foo   bar");
        let res = parser.parse(input);
        assert!(res.is_err());
    }
}
