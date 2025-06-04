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

    // TODO: write a test for this parser
    fn comment(input: Span) -> Result<()> {
        value((), (tag("#"), not_line_ending, line_ending.or(eof))).parse(input)
    }

    pub fn end_of_line(input: Span) -> Result<Span> {
        recognize(many1((
            linebreak.or(comment).or(end_of_file),
            inline_whitespace,
        )))
        .parse(input)
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

    pub fn and(input: Span) -> Result<Span> {
        token(tag("and")).parse(input)
    }

    pub fn as_(input: Span) -> Result<Span> {
        token(tag("as")).parse(input)
    }

    pub fn false_(input: Span) -> Result<Span> {
        token(tag("false")).parse(input)
    }

    pub fn from(input: Span) -> Result<Span> {
        token(tag("from")).parse(input)
    }

    pub fn if_(input: Span) -> Result<Span> {
        token(tag("if")).parse(input)
    }

    pub fn import(input: Span) -> Result<Span> {
        token(tag("import")).parse(input)
    }

    pub fn not(input: Span) -> Result<Span> {
        token(tag("not")).parse(input)
    }

    pub fn or(input: Span) -> Result<Span> {
        token(tag("or")).parse(input)
    }

    pub fn true_(input: Span) -> Result<Span> {
        token(tag("true")).parse(input)
    }

    pub fn section(input: Span) -> Result<Span> {
        token(tag("section")).parse(input)
    }

    pub fn test(input: Span) -> Result<Span> {
        token(tag("test")).parse(input)
    }

    pub fn use_(input: Span) -> Result<Span> {
        token(tag("use")).parse(input)
    }
}

pub mod symbol {
    use nom::{Parser as _, bytes::complete::tag};

    use super::{Result, Span, util::token};

    pub fn bang_equals(input: Span) -> Result<Span> {
        token(tag("!=")).parse(input)
    }

    pub fn bar(input: Span) -> Result<Span> {
        token(tag("|")).parse(input)
    }

    pub fn brace_left(input: Span) -> Result<Span> {
        token(tag("{")).parse(input)
    }

    pub fn brace_right(input: Span) -> Result<Span> {
        token(tag("}")).parse(input)
    }

    pub fn bracket_left(input: Span) -> Result<Span> {
        token(tag("[")).parse(input)
    }

    pub fn bracket_right(input: Span) -> Result<Span> {
        token(tag("]")).parse(input)
    }

    pub fn caret(input: Span) -> Result<Span> {
        token(tag("^")).parse(input)
    }

    pub fn colon(input: Span) -> Result<Span> {
        token(tag(":")).parse(input)
    }

    pub fn comma(input: Span) -> Result<Span> {
        token(tag(",")).parse(input)
    }

    pub fn dollar(input: Span) -> Result<Span> {
        token(tag("$")).parse(input)
    }

    pub fn dot(input: Span) -> Result<Span> {
        token(tag(".")).parse(input)
    }

    pub fn equals(input: Span) -> Result<Span> {
        token(tag("=")).parse(input)
    }

    pub fn equals_equals(input: Span) -> Result<Span> {
        token(tag("==")).parse(input)
    }

    pub fn greater_than(input: Span) -> Result<Span> {
        token(tag(">")).parse(input)
    }

    pub fn greater_than_equals(input: Span) -> Result<Span> {
        token(tag(">=")).parse(input)
    }

    pub fn less_than(input: Span) -> Result<Span> {
        token(tag("<")).parse(input)
    }

    pub fn less_than_equals(input: Span) -> Result<Span> {
        token(tag("<=")).parse(input)
    }

    pub fn minus(input: Span) -> Result<Span> {
        token(tag("-")).parse(input)
    }

    pub fn minus_minus(input: Span) -> Result<Span> {
        token(tag("--")).parse(input)
    }

    pub fn paren_left(input: Span) -> Result<Span> {
        token(tag("(")).parse(input)
    }

    pub fn paren_right(input: Span) -> Result<Span> {
        token(tag(")")).parse(input)
    }

    pub fn percent(input: Span) -> Result<Span> {
        token(tag("%")).parse(input)
    }

    pub fn plus(input: Span) -> Result<Span> {
        token(tag("+")).parse(input)
    }

    pub fn star(input: Span) -> Result<Span> {
        token(tag("*")).parse(input)
    }

    pub fn star_star(input: Span) -> Result<Span> {
        token(tag("**")).parse(input)
    }

    pub fn slash(input: Span) -> Result<Span> {
        token(tag("/")).parse(input)
    }

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

    pub fn identifier(input: Span) -> Result<Span> {
        token((
            satisfy(|c: char| c.is_alphabetic() || c == '_'),
            take_while(|c: char| c.is_alphanumeric() || c == '_'),
        ))
        .parse(input)
    }

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
