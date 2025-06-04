use super::util::{Parser, Result, Span};

mod util {
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
}

pub mod structure {
    use nom::{
        Parser as _,
        bytes::complete::tag,
        character::complete::{line_ending, not_line_ending},
        combinator::{eof, recognize, value},
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
        value((), (tag("#"), not_line_ending, line_ending.or(eof))).parse(input)
    }

    /// Parses one or more linebreaks, comments, or end-of-file markers, including trailing whitespace.
    pub fn end_of_line(input: Span) -> Result<Span> {
        recognize(many1((
            linebreak.or(comment).or(end_of_file),
            inline_whitespace,
        )))
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
}

mod note {
    use nom::character::complete::line_ending;
    use nom::combinator::{cut, verify};
    use nom::multi::many0;
    use nom::sequence::terminated;
    use nom::{
        Parser as _, bytes::complete::tag, character::complete::not_line_ending,
        combinator::recognize,
    };

    use super::{Result, Span, structure::end_of_line, util::inline_whitespace};

    fn single_line_note(input: Span) -> Result<Span> {
        terminated(recognize((tag("~"), not_line_ending)), end_of_line).parse(input)
    }

    fn multi_line_note_delimiter(input: Span) -> Result<Span> {
        recognize((
            inline_whitespace,
            tag("~~~"),
            many0(tag("~")),
            inline_whitespace,
        ))
        .parse(input)
    }

    fn multi_line_note_content(input: Span) -> Result<Span> {
        verify(recognize(many0((not_line_ending, line_ending))), |s| {
            // TODO: this allows for a content line to contain something like
            //       `~~~foo`, which we would want to disallow (I think)
            multi_line_note_delimiter(*s).is_err()
        })
        .parse(input)
    }

    fn multi_line_note(input: Span) -> Result<Span> {
        // TODO(error): add a note in the error that this failure is due to an
        //              unclosed multi-line note
        terminated(
            recognize((
                multi_line_note_delimiter,
                cut((
                    line_ending,
                    multi_line_note_content,
                    multi_line_note_delimiter,
                )),
            )),
            end_of_line,
        )
        .parse(input)
    }

    /// Parses a single-line or multi-line note, returning the note span.
    pub fn note(input: Span) -> Result<Span> {
        single_line_note.or(multi_line_note).parse(input)
    }
}

pub use note::note;

pub mod keyword {
    use nom::{Parser as _, bytes::complete::tag};

    use super::{Result, Span, util::token};

    const KEYWORDS: &[&str] = &[
        "and", "as", "false", "from", "if", "import", "not", "or", "true", "section", "test", "use",
    ];

    /// Parses the 'and' keyword token.
    pub fn and(input: Span) -> Result<Span> {
        token(tag("and")).parse(input)
    }

    /// Parses the 'as' keyword token.
    pub fn as_(input: Span) -> Result<Span> {
        token(tag("as")).parse(input)
    }

    /// Parses the 'false' keyword token.
    pub fn false_(input: Span) -> Result<Span> {
        token(tag("false")).parse(input)
    }

    /// Parses the 'from' keyword token.
    pub fn from(input: Span) -> Result<Span> {
        token(tag("from")).parse(input)
    }

    /// Parses the 'if' keyword token.
    pub fn if_(input: Span) -> Result<Span> {
        token(tag("if")).parse(input)
    }

    /// Parses the 'import' keyword token.
    pub fn import(input: Span) -> Result<Span> {
        token(tag("import")).parse(input)
    }

    /// Parses the 'not' keyword token.
    pub fn not(input: Span) -> Result<Span> {
        token(tag("not")).parse(input)
    }

    /// Parses the 'or' keyword token.
    pub fn or(input: Span) -> Result<Span> {
        token(tag("or")).parse(input)
    }

    /// Parses the 'true' keyword token.
    pub fn true_(input: Span) -> Result<Span> {
        token(tag("true")).parse(input)
    }

    /// Parses the 'section' keyword token.
    pub fn section(input: Span) -> Result<Span> {
        token(tag("section")).parse(input)
    }

    /// Parses the 'test' keyword token.
    pub fn test(input: Span) -> Result<Span> {
        token(tag("test")).parse(input)
    }

    /// Parses the 'use' keyword token.
    pub fn use_(input: Span) -> Result<Span> {
        token(tag("use")).parse(input)
    }
}

pub mod symbol {
    use nom::{Parser as _, bytes::complete::tag};

    use super::{Result, Span, util::token};

    /// Parses the '!=' symbol token.
    pub fn bang_equals(input: Span) -> Result<Span> {
        token(tag("!=")).parse(input)
    }

    /// Parses the '|' symbol token.
    pub fn bar(input: Span) -> Result<Span> {
        token(tag("|")).parse(input)
    }

    /// Parses the '{' symbol token.
    pub fn brace_left(input: Span) -> Result<Span> {
        token(tag("{")).parse(input)
    }

    /// Parses the '}' symbol token.
    pub fn brace_right(input: Span) -> Result<Span> {
        token(tag("}")).parse(input)
    }

    /// Parses the '[' symbol token.
    pub fn bracket_left(input: Span) -> Result<Span> {
        token(tag("[")).parse(input)
    }

    /// Parses the ']' symbol token.
    pub fn bracket_right(input: Span) -> Result<Span> {
        token(tag("]")).parse(input)
    }

    /// Parses the '^' symbol token.
    pub fn caret(input: Span) -> Result<Span> {
        token(tag("^")).parse(input)
    }

    /// Parses the ':' symbol token.
    pub fn colon(input: Span) -> Result<Span> {
        token(tag(":")).parse(input)
    }

    /// Parses the ',' symbol token.
    pub fn comma(input: Span) -> Result<Span> {
        token(tag(",")).parse(input)
    }

    /// Parses the '$' symbol token.
    pub fn dollar(input: Span) -> Result<Span> {
        token(tag("$")).parse(input)
    }

    /// Parses the '.' symbol token.
    pub fn dot(input: Span) -> Result<Span> {
        token(tag(".")).parse(input)
    }

    /// Parses the '=' symbol token.
    pub fn equals(input: Span) -> Result<Span> {
        token(tag("=")).parse(input)
    }

    /// Parses the '==' symbol token.
    pub fn equals_equals(input: Span) -> Result<Span> {
        token(tag("==")).parse(input)
    }

    /// Parses the '>' symbol token.
    pub fn greater_than(input: Span) -> Result<Span> {
        token(tag(">")).parse(input)
    }

    /// Parses the '>=' symbol token.
    pub fn greater_than_equals(input: Span) -> Result<Span> {
        token(tag(">=")).parse(input)
    }

    /// Parses the '<' symbol token.
    pub fn less_than(input: Span) -> Result<Span> {
        token(tag("<")).parse(input)
    }

    /// Parses the '<=' symbol token.
    pub fn less_than_equals(input: Span) -> Result<Span> {
        token(tag("<=")).parse(input)
    }

    /// Parses the '-' symbol token.
    pub fn minus(input: Span) -> Result<Span> {
        token(tag("-")).parse(input)
    }

    /// Parses the '--' symbol token.
    pub fn minus_minus(input: Span) -> Result<Span> {
        token(tag("--")).parse(input)
    }

    /// Parses the '(' symbol token.
    pub fn paren_left(input: Span) -> Result<Span> {
        token(tag("(")).parse(input)
    }

    /// Parses the ')' symbol token.
    pub fn paren_right(input: Span) -> Result<Span> {
        token(tag(")")).parse(input)
    }

    /// Parses the '%' symbol token.
    pub fn percent(input: Span) -> Result<Span> {
        token(tag("%")).parse(input)
    }

    /// Parses the '+' symbol token.
    pub fn plus(input: Span) -> Result<Span> {
        token(tag("+")).parse(input)
    }

    /// Parses the '*' symbol token.
    pub fn star(input: Span) -> Result<Span> {
        token(tag("*")).parse(input)
    }

    /// Parses the '**' symbol token.
    pub fn star_star(input: Span) -> Result<Span> {
        token(tag("**")).parse(input)
    }

    /// Parses the '/' symbol token.
    pub fn slash(input: Span) -> Result<Span> {
        token(tag("/")).parse(input)
    }

    /// Parses the '//' symbol token.
    pub fn slash_slash(input: Span) -> Result<Span> {
        token(tag("//")).parse(input)
    }
}

pub mod literal {
    use nom::{
        Parser as _,
        bytes::complete::take_while,
        character::complete::{char, digit1},
        combinator::{cut, opt},
    };

    use super::{Result, Span, util::token};

    /// Parses a number literal, supporting optional sign, decimal, and exponent.
    pub fn number(input: Span) -> Result<Span> {
        let sign1 = opt(char('+').or(char('-')));
        let sign2 = opt(char('+').or(char('-')));
        let e = opt(char('e').or(char('E')));
        token((
            opt(sign1),
            digit1,
            opt((char('.'), cut(digit1))),
            opt((e, cut((sign2, digit1)))),
        ))
        .parse(input)
    }

    /// Parses a string literal delimited by double quotes.
    pub fn string(input: Span) -> Result<Span> {
        token((
            char('"'),
            cut((take_while(|c: char| c != '"' && c != '\n'), char('"'))),
        ))
        .parse(input)
    }
}

pub mod naming {
    use nom::{Parser as _, bytes::complete::take_while, character::complete::satisfy};

    use super::{Result, Span, util::token};

    /// Parses an identifier (alphabetic or underscore, then alphanumeric or underscore).
    pub fn identifier(input: Span) -> Result<Span> {
        token((
            satisfy(|c: char| c.is_alphabetic() || c == '_'),
            take_while(|c: char| c.is_alphanumeric() || c == '_'),
        ))
        .parse(input)
    }

    /// Parses a label (alphabetic or underscore, then alphanumeric, underscore, dash, space, or tab).
    pub fn label(input: Span) -> Result<Span> {
        token((
            satisfy(|c: char| c.is_alphabetic() || c == '_'),
            take_while(|c: char| {
                c.is_alphanumeric() || c == '_' || c == '-' || c == ' ' || c == '\t'
            }),
        ))
        .parse(input)
    }
}
