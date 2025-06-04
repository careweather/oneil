use nom::{
    Parser as _,
    character::complete::{char, line_ending, not_line_ending},
    combinator::{eof, opt, recognize, value},
    multi::many1,
};

use crate::parser::token::util::inline_whitespace;

use super::{Result, Span};

fn linebreak(input: Span) -> Result<()> {
    value((), line_ending).parse(input)
}

fn end_of_file(input: Span) -> Result<()> {
    value((), eof).parse(input)
}

fn comment(input: Span) -> Result<()> {
    value((), (char('#'), not_line_ending, line_ending.or(eof))).parse(input)
}

/// Parses one or more linebreaks, comments, or end-of-file markers, including trailing whitespace.
pub fn end_of_line(input: Span) -> Result<Span> {
    recognize(
        (
            many1((linebreak.or(comment), inline_whitespace)),
            opt(end_of_file),
        )
            .map(|_| ())
            .or(end_of_file),
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_end_of_file() {
        // End of file
        let input = Span::new("");
        let (rest, _) = end_of_file(input).expect("should parse end of file");
        assert_eq!(rest.fragment(), &"");

        // Not end of file
        let input = Span::new("not empty");
        let res = end_of_file(input);
        assert!(
            res.is_err(),
            "should not parse non-empty input as end of file"
        );
    }

    #[test]
    fn test_comment() {
        // Single comment with newline
        let input = Span::new("# this is a comment\nrest");
        let (rest, _) = comment(input).expect("should parse comment");
        assert_eq!(rest.fragment(), &"rest");

        // Single comment at EOF
        let input = Span::new("# only comment");
        let (rest, _) = comment(input).expect("should parse comment at EOF");
        assert_eq!(rest.fragment(), &"");

        // Not a comment
        let input = Span::new("not a comment");
        let res = comment(input);
        assert!(res.is_err());
    }

    #[test]
    fn test_end_of_line() {
        // One linebreak
        let input = Span::new("\nrest");
        let (rest, matched) = end_of_line(input).expect("should parse linebreak");
        assert_eq!(rest.fragment(), &"rest");
        assert!(matched.trim().is_empty());

        // One comment
        let input = Span::new("# comment\nrest");
        let (rest, matched) = end_of_line(input).expect("should parse comment as end_of_line");
        assert_eq!(rest.fragment(), &"rest");
        assert!(matched.contains("# comment"));

        // Multiple linebreaks and comments
        let input = Span::new("\n# foo\n\n# bar\nrest");
        let (rest, matched) = end_of_line(input).expect("should parse multiple end_of_line");
        assert_eq!(rest.fragment(), &"rest");
        assert!(matched.contains("# foo"));
        assert!(matched.contains("# bar"));

        // End of file
        let input = Span::new("");
        let (rest, matched) = end_of_line(input).expect("should parse EOF as end_of_line");
        assert_eq!(rest.fragment(), &"");
        assert!(matched.is_empty() || matched.trim().is_empty());

        // Multiple linebreaks, comments, and EOF
        let input = Span::new("\n# comment\n\n");
        let (rest, matched) =
            end_of_line(input).expect("should parse multiple end_of_line with EOF");
        assert_eq!(rest.fragment(), &"");
        assert!(matched.contains("# comment"));
    }
}
