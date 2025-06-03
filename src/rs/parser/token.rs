use super::util::{Parser, Result, Span};

mod util {
    use nom::{
        Parser as _,
        character::complete::space0,
        combinator::{recognize, value},
        sequence::terminated,
    };

    use super::{Parser, Result, Span};

    pub fn inline_whitespace<'a>(input: Span<'a>) -> Result<'a, ()> {
        value((), space0).parse(input)
    }

    pub fn token<'a, F, O>(f: F) -> impl Parser<'a, Span<'a>>
    where
        F: Parser<'a, O>,
    {
        terminated(recognize(f), inline_whitespace)
    }
}

mod structure {
    use nom::{
        Parser as _,
        bytes::complete::tag,
        character::complete::{line_ending, not_line_ending},
        combinator::{eof, recognize, value},
        multi::many1,
    };

    use crate::parser::token::util::inline_whitespace;

    use super::{Result, Span};

    fn linebreak<'a>(input: Span<'a>) -> Result<'a, ()> {
        value((), line_ending).parse(input)
    }

    fn end_of_file<'a>(input: Span<'a>) -> Result<'a, ()> {
        value((), eof).parse(input)
    }

    // TODO: write a test for this parser
    fn comment<'a>(input: Span<'a>) -> Result<'a, ()> {
        value((), (tag("#"), not_line_ending, line_ending.or(eof))).parse(input)
    }

    pub fn end_of_line<'a>(input: Span<'a>) -> Result<'a, Span<'a>> {
        recognize(many1((
            linebreak.or(comment).or(end_of_file),
            inline_whitespace,
        )))
        .parse(input)
    }
}

// NOTES
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

    fn single_line_note<'a>(input: Span<'a>) -> Result<'a, Span<'a>> {
        terminated(recognize((tag("~"), not_line_ending)), end_of_line).parse(input)
    }

    fn multi_line_note_delimiter<'a>(input: Span<'a>) -> Result<'a, Span<'a>> {
        recognize((
            inline_whitespace,
            tag("~~~"),
            many0(tag("~")),
            inline_whitespace,
        ))
        .parse(input)
    }

    fn multi_line_note_content<'a>(input: Span<'a>) -> Result<'a, Span<'a>> {
        verify(recognize(many0((not_line_ending, line_ending))), |s| {
            // TODO: this allows for a content line to contain something like
            //       `~~~foo`, which we would want to disallow (I think)
            multi_line_note_delimiter(*s).is_err()
        })
        .parse(input)
    }

    fn multi_line_note<'a>(input: Span<'a>) -> Result<'a, Span<'a>> {
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

    pub fn note<'a>(input: Span<'a>) -> Result<'a, Span<'a>> {
        single_line_note.or(multi_line_note).parse(input)
    }
}

pub use note::note;

// KEYWORDS
mod keyword {}
