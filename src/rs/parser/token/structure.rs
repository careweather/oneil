//! Provides parsers for structural elements in the Oneil language.
//!
//! This module currently contains parsers for end-of-line tokens, which include:
//! - Line breaks (`\n`)
//! - Comments (starting with `#` and continuing to end of line)
//! - End-of-file markers
//! - Any combination of the above with optional whitespace
//!
//! # Examples
//!
//! ```
//! use oneil::parser::token::structure::end_of_line;
//! use oneil::parser::{Config, Span};
//!
//! // Parse a comment followed by newline
//! let input = Span::new_extra("# This is a comment\nrest", Config::default());
//! let (rest, matched) = end_of_line(input).unwrap();
//! assert_eq!(rest.fragment(), &"rest");
//!
//! // Parse multiple blank lines
//! let input = Span::new_extra("\n\n\nrest", Config::default());
//! let (rest, matched) = end_of_line(input).unwrap();
//! assert_eq!(rest.fragment(), &"rest");
//!
//! // Parse end of file
//! let input = Span::new_extra("", Config::default());
//! let (rest, matched) = end_of_line(input).unwrap();
//! assert_eq!(rest.fragment(), &"");
//! ```

use nom::{
    Parser as _,
    character::complete::{char, line_ending, not_line_ending},
    combinator::{eof, opt, recognize},
    multi::many0,
};

use super::{
    Result, Span,
    error::{ErrorHandlingParser, TokenError},
    util::{Token, inline_whitespace},
};

fn linebreak(input: Span) -> Result<Span, TokenError> {
    line_ending.parse(input)
}

fn end_of_file(input: Span) -> Result<Span, TokenError> {
    eof.parse(input)
}

fn comment(input: Span) -> Result<Span, TokenError> {
    recognize((char('#'), not_line_ending, line_ending.or(eof))).parse(input)
}

/// Parses one or more linebreaks, comments, or end-of-file markers, including
/// trailing whitespace
pub fn end_of_line(input: Span) -> Result<Token, TokenError> {
    let (rest, first_line_break) = linebreak
        .or(comment)
        .or(end_of_file)
        .map_error(TokenError::expected_end_of_line)
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
    use crate::parser::Config;

    #[test]
    fn test_end_of_file_empty() {
        let input = Span::new_extra("", Config::default());
        let (rest, _) = end_of_file(input).expect("should parse end of file");
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_end_of_file_not_empty() {
        let input = Span::new_extra("not empty", Config::default());
        let res = end_of_file(input);
        assert!(
            res.is_err(),
            "should not parse non-empty input as end of file"
        );
    }

    #[test]
    fn test_comment_with_newline() {
        let input = Span::new_extra("# this is a comment\nrest", Config::default());
        let (rest, _) = comment(input).expect("should parse comment");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_comment_at_eof() {
        let input = Span::new_extra("# only comment", Config::default());
        let (rest, _) = comment(input).expect("should parse comment at EOF");
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_comment_invalid() {
        let input = Span::new_extra("not a comment", Config::default());
        let res = comment(input);
        assert!(res.is_err());
    }

    #[test]
    fn test_end_of_line_single_linebreak() {
        let input = Span::new_extra("\nrest", Config::default());
        let (rest, matched) = end_of_line(input).expect("should parse linebreak");
        assert_eq!(rest.fragment(), &"rest");
        assert_eq!(matched.lexeme(), "\n");
        assert!(matched.whitespace().is_empty());
    }

    #[test]
    fn test_end_of_line_single_comment() {
        let input = Span::new_extra("# comment\nrest", Config::default());
        let (rest, matched) = end_of_line(input).expect("should parse comment as end_of_line");
        assert_eq!(rest.fragment(), &"rest");
        assert_eq!(matched.lexeme(), "# comment\n");
        assert!(matched.whitespace().is_empty());
    }

    #[test]
    fn test_end_of_line_multiple() {
        let input = Span::new_extra("\n# foo\n\n# bar\nrest", Config::default());
        let (rest, matched) = end_of_line(input).expect("should parse multiple end_of_line");
        assert_eq!(rest.fragment(), &"rest");
        assert_eq!(matched.lexeme(), "\n");
        assert!(matched.whitespace().contains("# foo\n"));
        assert!(matched.whitespace().contains("# bar\n"));
    }

    #[test]
    fn test_end_of_line_eof() {
        let input = Span::new_extra("", Config::default());
        let (rest, matched) = end_of_line(input).expect("should parse EOF as end_of_line");
        assert_eq!(rest.fragment(), &"");
        assert_eq!(matched.lexeme(), "");
        assert_eq!(matched.whitespace(), "");
    }

    #[test]
    fn test_end_of_line_multiple_with_eof() {
        let input = Span::new_extra("\n# comment\n\n", Config::default());
        let (rest, matched) =
            end_of_line(input).expect("should parse multiple end_of_line with EOF");
        assert_eq!(rest.fragment(), &"");
        assert_eq!(matched.lexeme(), "\n");
        assert!(matched.whitespace().contains("# comment\n"));
    }
}
