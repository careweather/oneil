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

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_inline_whitespace() {
            let input = Span::new("   abc");
            let (rest, _) = inline_whitespace(input).expect("should parse leading spaces");
            assert_eq!(rest.fragment(), &"abc");

            let input = Span::new("\t\tfoo");
            let (rest, _) = inline_whitespace(input).expect("should parse leading tabs");
            assert_eq!(rest.fragment(), &"foo");

            let input = Span::new("bar");
            let (rest, _) = inline_whitespace(input).expect("should parse no whitespace");
            assert_eq!(rest.fragment(), &"bar");
        }

        #[test]
        fn test_token() {
            use nom::bytes::complete::tag;
            // token should parse a tag and trailing whitespace
            let mut parser = token(tag("foo"));
            let input = Span::new("foo   bar");
            let (rest, matched) = parser
                .parse(input)
                .expect("should parse token with trailing whitespace");
            assert_eq!(matched.fragment(), &"foo");
            assert_eq!(rest.fragment(), &"bar");

            // token should not consume if tag does not match
            let mut parser = token(tag("baz"));
            let input = Span::new("foo   bar");
            let res = parser.parse(input);
            assert!(res.is_err());
        }
    }
}

pub mod structure {
    use nom::{
        Parser as _,
        bytes::complete::tag,
        character::complete::{line_ending, not_line_ending},
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
        value((), (tag("#"), not_line_ending, line_ending.or(eof))).parse(input)
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
}

mod note {
    use nom::bytes::complete::take_while;
    use nom::character::complete::line_ending;
    use nom::combinator::{cut, verify};
    use nom::multi::many0;
    use nom::sequence::terminated;
    use nom::{
        Parser as _,
        bytes::complete::tag,
        character::complete::{char, not_line_ending},
        combinator::recognize,
    };

    use super::{Result, Span, structure::end_of_line, util::inline_whitespace};

    /// Parses a single-line note, which starts with `~` and ends with a newline.
    ///
    /// The note can contain any characters except for a newline, and it must be
    /// followed by a newline to be considered valid.
    pub fn single_line_note(input: Span) -> Result<Span> {
        terminated(recognize((tag("~"), not_line_ending)), end_of_line).parse(input)
    }

    fn multi_line_note_delimiter(input: Span) -> Result<Span> {
        recognize((
            inline_whitespace,
            verify(take_while(|c: char| c == '~'), |s: &Span| s.len() >= 3),
            inline_whitespace,
        ))
        .parse(input)
    }

    fn multi_line_note_content(input: Span) -> Result<Span> {
        recognize(many0(verify((not_line_ending, line_ending), |(s, _)| {
            multi_line_note_delimiter.parse(*s).is_err()
        })))
        .parse(input)
    }

    /// Parses a multi-line note, which starts and ends with `~~~` and can contain
    /// multiple lines of content, each ending with a newline.
    ///
    /// The content must not contain the multi-line note delimiter `~~~` on its own
    /// line, and the note must be closed with a matching `~~~` delimiter.
    ///
    /// If the multi-line note is not closed properly, this parser will fail.
    pub fn multi_line_note(input: Span) -> Result<Span> {
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

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_single_line_note() {
            // Single line note with newline
            let input = Span::new("~ this is a note\nrest");
            let (rest, matched) = single_line_note(input).expect("should parse single line note");
            assert_eq!(matched.fragment(), &"~ this is a note");
            assert_eq!(rest.fragment(), &"rest");

            // Single line note at EOF
            let input = Span::new("~ note");
            let (rest, matched) =
                single_line_note(input).expect("should parse single line note at EOF");
            assert_eq!(matched.fragment(), &"~ note");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_multi_line_note() {
            // Multi-line note with content and closing delimiter
            let input = Span::new("~~~\nThis is a multi-line note.\nSecond line.\n~~~\nrest");
            let (rest, matched) = multi_line_note(input).expect("should parse multi-line note");
            assert!(matched.fragment().contains("This is a multi-line note."));
            assert!(matched.fragment().contains("Second line."));
            assert_eq!(rest.fragment(), &"rest");

            // Multi-line note with extra tildes in delimiter
            let input = Span::new("~~~~~\nfoo\nbar\n~~~~~\nrest");
            let (rest, matched) =
                multi_line_note(input).expect("should parse multi-line note with extra tildes");
            assert!(matched.fragment().contains("foo"));
            assert!(matched.fragment().contains("bar"));
            assert_eq!(rest.fragment(), &"rest");

            // Empty multi-line note
            let input = Span::new("~~~\n~~~\nrest");
            let (rest, _) = multi_line_note(input).expect("should parse empty multi-line note");
            assert_eq!(rest.fragment(), &"rest");

            // Multi-line note not closed
            let input = Span::new("~~~\nUnclosed note\n");
            let res = multi_line_note(input);
            assert!(res.is_err(), "should not parse unclosed multi-line note");
        }
    }
}

pub mod keyword {
    use nom::{Parser as _, bytes::complete::tag, character::complete::satisfy, combinator::peek};

    use super::{Parser, Result, Span, util::token};

    const KEYWORDS: &[&str] = &[
        "and", "as", "false", "from", "if", "import", "not", "or", "true", "section", "test", "use",
    ];

    fn keyword(kw_str: &str) -> impl Parser<Span> {
        token((
            tag(kw_str),
            peek(satisfy(|c: char| !c.is_alphanumeric() && c != '_')),
        ))
    }

    /// Parses the 'and' keyword token.
    pub fn and(input: Span) -> Result<Span> {
        keyword("and").parse(input)
    }

    /// Parses the 'as' keyword token.
    pub fn as_(input: Span) -> Result<Span> {
        keyword("as").parse(input)
    }

    /// Parses the 'false' keyword token.
    pub fn false_(input: Span) -> Result<Span> {
        keyword("false").parse(input)
    }

    /// Parses the 'from' keyword token.
    pub fn from(input: Span) -> Result<Span> {
        keyword("from").parse(input)
    }

    /// Parses the 'if' keyword token.
    pub fn if_(input: Span) -> Result<Span> {
        keyword("if").parse(input)
    }

    /// Parses the 'import' keyword token.
    pub fn import(input: Span) -> Result<Span> {
        keyword("import").parse(input)
    }

    /// Parses the 'not' keyword token.
    pub fn not(input: Span) -> Result<Span> {
        keyword("not").parse(input)
    }

    /// Parses the 'or' keyword token.
    pub fn or(input: Span) -> Result<Span> {
        keyword("or").parse(input)
    }

    /// Parses the 'true' keyword token.
    pub fn true_(input: Span) -> Result<Span> {
        keyword("true").parse(input)
    }

    /// Parses the 'section' keyword token.
    pub fn section(input: Span) -> Result<Span> {
        keyword("section").parse(input)
    }

    /// Parses the 'test' keyword token.
    pub fn test(input: Span) -> Result<Span> {
        keyword("test").parse(input)
    }

    /// Parses the 'use' keyword token.
    pub fn use_(input: Span) -> Result<Span> {
        keyword("use").parse(input)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::parser::token::Span;

        #[test]
        fn test_and() {
            let input = Span::new("and rest");
            let (rest, matched) = and(input).expect("should parse 'and' keyword");
            assert_eq!(matched.fragment(), &"and");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_as() {
            let input = Span::new("as foo");
            let (rest, matched) = as_(input).expect("should parse 'as' keyword");
            assert_eq!(matched.fragment(), &"as");
            assert_eq!(rest.fragment(), &"foo");
        }

        #[test]
        fn test_false() {
            let input = Span::new("false true");
            let (rest, matched) = false_(input).expect("should parse 'false' keyword");
            assert_eq!(matched.fragment(), &"false");
            assert_eq!(rest.fragment(), &"true");
        }

        #[test]
        fn test_from() {
            let input = Span::new("from bar");
            let (rest, matched) = from(input).expect("should parse 'from' keyword");
            assert_eq!(matched.fragment(), &"from");
            assert_eq!(rest.fragment(), &"bar");
        }

        #[test]
        fn test_if() {
            let input = Span::new("if baz");
            let (rest, matched) = if_(input).expect("should parse 'if' keyword");
            assert_eq!(matched.fragment(), &"if");
            assert_eq!(rest.fragment(), &"baz");
        }

        #[test]
        fn test_import() {
            let input = Span::new("import foo");
            let (rest, matched) = import(input).expect("should parse 'import' keyword");
            assert_eq!(matched.fragment(), &"import");
            assert_eq!(rest.fragment(), &"foo");
        }

        #[test]
        fn test_not() {
            let input = Span::new("not bar");
            let (rest, matched) = not(input).expect("should parse 'not' keyword");
            assert_eq!(matched.fragment(), &"not");
            assert_eq!(rest.fragment(), &"bar");
        }

        #[test]
        fn test_or() {
            let input = Span::new("or baz");
            let (rest, matched) = or(input).expect("should parse 'or' keyword");
            assert_eq!(matched.fragment(), &"or");
            assert_eq!(rest.fragment(), &"baz");
        }

        #[test]
        fn test_true() {
            let input = Span::new("true false");
            let (rest, matched) = true_(input).expect("should parse 'true' keyword");
            assert_eq!(matched.fragment(), &"true");
            assert_eq!(rest.fragment(), &"false");
        }

        #[test]
        fn test_section() {
            let input = Span::new("section test");
            let (rest, matched) = section(input).expect("should parse 'section' keyword");
            assert_eq!(matched.fragment(), &"section");
            assert_eq!(rest.fragment(), &"test");
        }

        #[test]
        fn test_test() {
            let input = Span::new("test use");
            let (rest, matched) = test(input).expect("should parse 'test' keyword");
            assert_eq!(matched.fragment(), &"test");
            assert_eq!(rest.fragment(), &"use");
        }

        #[test]
        fn test_use() {
            let input = Span::new("use foo");
            let (rest, matched) = use_(input).expect("should parse 'use' keyword");
            assert_eq!(matched.fragment(), &"use");
            assert_eq!(rest.fragment(), &"foo");
        }

        #[test]
        fn test_keyword_with_trailing_whitespace() {
            let input = Span::new("and   foo");
            let (rest, matched) = and(input).expect("should parse 'and' with trailing whitespace");
            assert_eq!(matched.fragment(), &"and");
            assert_eq!(rest.fragment(), &"foo");
        }

        #[test]
        fn test_keyword_not_at_start() {
            let input = Span::new("foo and bar");
            let res = and(input);
            assert!(res.is_err(), "should not parse 'and' if not at start");
        }

        #[test]
        fn test_keyword_prefix() {
            let input = Span::new("anderson");
            let res = and(input);
            assert!(res.is_err(), "should not parse 'and' as prefix");
        }
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
        let e = char('e').or(char('E'));
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

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::parser::token::Span;

        #[test]
        fn test_number() {
            // Integer
            let input = Span::new("42 rest");
            let (rest, matched) = number(input).expect("should parse integer");
            assert_eq!(matched.fragment(), &"42");
            assert_eq!(rest.fragment(), &"rest");

            // Negative integer
            let input = Span::new("-17 rest");
            let (rest, matched) = number(input).expect("should parse negative integer");
            assert_eq!(matched.fragment(), &"-17");
            assert_eq!(rest.fragment(), &"rest");

            // Decimal
            let input = Span::new("3.1415 rest");
            let (rest, matched) = number(input).expect("should parse decimal");
            assert_eq!(matched.fragment(), &"3.1415");
            assert_eq!(rest.fragment(), &"rest");

            // Exponent
            let input = Span::new("2.5e10 rest");
            let (rest, matched) = number(input).expect("should parse exponent");
            assert_eq!(matched.fragment(), &"2.5e10");
            assert_eq!(rest.fragment(), &"rest");

            // Negative exponent
            let input = Span::new("-1.2E-3 rest");
            let (rest, matched) = number(input).expect("should parse negative exponent");
            assert_eq!(matched.fragment(), &"-1.2E-3");
            assert_eq!(rest.fragment(), &"rest");

            // Not a number
            let input = Span::new("foo");
            let res = number(input);
            assert!(res.is_err());
        }

        #[test]
        fn test_string() {
            // Simple string
            let input = Span::new("\"hello\" rest");
            let (rest, matched) = string(input).expect("should parse string");
            assert_eq!(matched.fragment(), &"\"hello\"");
            assert_eq!(rest.fragment(), &"rest");

            // String with spaces
            let input = Span::new("\"foo bar\" baz");
            let (rest, matched) = string(input).expect("should parse string with spaces");
            assert_eq!(matched.fragment(), &"\"foo bar\"");
            assert_eq!(rest.fragment(), &"baz");

            // String doesn't support escape sequences
            let input = Span::new("\"foo \\\" bar");
            let (rest, matched) =
                string(input).expect("should parse string (escape sequences not supported)");
            assert_eq!(matched.fragment(), &"\"foo \\\"");
            assert_eq!(rest.fragment(), &"bar");

            // Unterminated string
            let input = Span::new("\"unterminated");
            let res = string(input);
            assert!(res.is_err(), "should not parse unterminated string");
        }
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

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::parser::token::Span;

        #[test]
        fn test_identifier_basic() {
            let input = Span::new("foo rest");
            let (rest, matched) = identifier(input).expect("should parse basic identifier");
            assert_eq!(matched.fragment(), &"foo");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_identifier_underscore() {
            let input = Span::new("_foo123 bar");
            let (rest, matched) =
                identifier(input).expect("should parse identifier with underscore");
            assert_eq!(matched.fragment(), &"_foo123");
            assert_eq!(rest.fragment(), &"bar");
        }

        #[test]
        fn test_identifier_invalid() {
            let input = Span::new("123abc");
            let res = identifier(input);
            assert!(
                res.is_err(),
                "should not parse identifier starting with digit"
            );
        }

        #[test]
        fn test_identifier_only_underscore() {
            let input = Span::new("_ rest");
            let (rest, matched) =
                identifier(input).expect("should parse single underscore identifier");
            assert_eq!(matched.fragment(), &"_");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_label_basic() {
            let input = Span::new("foo-bar: rest");
            let (rest, matched) = label(input).expect("should parse label with dash");
            assert_eq!(matched.fragment(), &"foo-bar");
            assert_eq!(rest.fragment(), &": rest");
        }

        #[test]
        fn test_label_with_spaces_and_tabs() {
            let input = Span::new("foo bar\tbaz: rest");
            let (rest, matched) = label(input).expect("should parse label with spaces and tabs");
            assert_eq!(matched.fragment(), &"foo bar\tbaz");
            assert_eq!(rest.fragment(), &": rest");
        }

        #[test]
        fn test_label_invalid_start() {
            let input = Span::new("-foo");
            let res = label(input);
            assert!(res.is_err(), "should not parse label starting with dash");
        }

        #[test]
        fn test_label_only_underscore() {
            let input = Span::new("_: rest");
            let (rest, matched) = label(input).expect("should parse label with only underscore");
            assert_eq!(matched.fragment(), &"_");
            assert_eq!(rest.fragment(), &": rest");
        }

        #[test]
        fn test_label_with_multiple_dashes() {
            let input = Span::new("foo-bar-baz: rest");
            let (rest, matched) = label(input).expect("should parse label with multiple dashes");
            assert_eq!(matched.fragment(), &"foo-bar-baz");
            assert_eq!(rest.fragment(), &": rest");
        }

        #[test]
        fn test_label_with_trailing_whitespace() {
            let input = Span::new("foo : rest");
            let (rest, matched) =
                label(input).expect("should parse label with trailing whitespace");
            assert_eq!(matched.fragment(), &"foo ");
            assert_eq!(rest.fragment(), &": rest");
        }
    }
}
