use oneil_error::{Context, ErrorLocation};

use crate::{
    error::reason::{ExpectKind, IncompleteKind, ParserErrorReason},
    token::error::{IncompleteKind as TokenIncompleteKind, TokenErrorKind},
};

pub fn from_source(
    offset: usize,
    reason: &ParserErrorReason,
    source: &str,
) -> Vec<(Context, Option<ErrorLocation>)> {
    let remaining_source = &source[offset..];

    [
        notes_only_allowed_at_start_of_model_or_section(reason, remaining_source),
        parameter_missing_label(reason, remaining_source),
        parameter_label_has_invalid_characters(reason, source, remaining_source, offset),
        string_literal_uses_double_quotes(remaining_source),
        decimal_literal_starts_with_dot(remaining_source),
        unclosed(reason, source),
        invalid_number_literal(reason, source),
    ]
    .into_iter()
    .flatten() // get rid of any None values
    .collect()
}

fn notes_only_allowed_at_start_of_model_or_section(
    reason: &ParserErrorReason,
    remaining_source: &str,
) -> Vec<(Context, Option<ErrorLocation>)> {
    let is_expect_decl = matches!(reason, ParserErrorReason::Expect(ExpectKind::Decl));
    if !is_expect_decl {
        return vec![];
    }

    let starts_with_tilde = remaining_source.starts_with('~');

    if starts_with_tilde {
        let message = "notes are only allowed at the beginning of model files and sections and after parameters and tests";
        vec![(Context::Note(message.to_string()), None)]
    } else {
        vec![]
    }
}

fn parameter_missing_label(
    reason: &ParserErrorReason,
    remaining_source: &str,
) -> Vec<(Context, Option<ErrorLocation>)> {
    let is_expect_decl = matches!(reason, ParserErrorReason::Expect(ExpectKind::Decl));
    if !is_expect_decl {
        return vec![];
    }

    let starts_with_ident_and_equals = parsers::ident_and_equals(remaining_source).is_ok();

    if starts_with_ident_and_equals {
        let message = "parameters must have a label";
        vec![(Context::Note(message.to_string()), None)]
    } else {
        vec![]
    }
}

fn parameter_label_has_invalid_characters(
    reason: &ParserErrorReason,
    source: &str,
    remaining_source: &str,
    remaining_source_offset: usize,
) -> Vec<(Context, Option<ErrorLocation>)> {
    let is_expect_decl = matches!(reason, ParserErrorReason::Expect(ExpectKind::Decl));

    if !is_expect_decl {
        return vec![];
    }

    let line = remaining_source
        .split_once('\n')
        .map(|(line, _)| line)
        .unwrap_or(remaining_source);

    let invalid_char_index = line
        .split_once('=')
        .map(|(before_equals, _)| before_equals)
        .and_then(|before_equals| before_equals.split_once(':'))
        .and_then(|(label, _)| {
            label.find(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-' && c != '\'')
        })
        .map(|index| index + remaining_source_offset);

    match invalid_char_index {
        Some(index) => {
            let note_message = "parameter labels must only contain the following characters: `a-z`, `A-Z`, `0-9`, `_`, `-`, `'`";

            let invalid_char_note_message = "invalid character found here";
            let invalid_char_location = ErrorLocation::from_source_and_offset(source, index);

            vec![
                (Context::Note(note_message.to_string()), None),
                (
                    Context::Note(invalid_char_note_message.to_string()),
                    Some(invalid_char_location),
                ),
            ]
        }
        None => {
            vec![]
        }
    }
}

fn string_literal_uses_double_quotes(
    remaining_source: &str,
) -> Vec<(Context, Option<ErrorLocation>)> {
    let starts_with_double_quote = remaining_source.starts_with('"');

    if starts_with_double_quote {
        let note_message = "string literals in Oneil use single quotes (`'`)";
        let help_message = "use single quotes (`'`) instead of double quotes (`\"`)";
        vec![
            (Context::Note(note_message.to_string()), None),
            (Context::Help(help_message.to_string()), None),
        ]
    } else {
        vec![]
    }
}

fn decimal_literal_starts_with_dot(
    remaining_source: &str,
) -> Vec<(Context, Option<ErrorLocation>)> {
    let starts_with_dot = remaining_source.starts_with('.');
    let is_followed_by_digit = remaining_source
        .chars()
        .skip(1) // skip the decimal point
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false);

    if starts_with_dot && is_followed_by_digit {
        let note_message = "decimal literals are not allowed to start with a `.`";
        let help_message = "add a leading `0`";
        vec![
            (Context::Note(note_message.to_string()), None),
            (Context::Help(help_message.to_string()), None),
        ]
    } else {
        vec![]
    }
}

fn unclosed(reason: &ParserErrorReason, source: &str) -> Vec<(Context, Option<ErrorLocation>)> {
    match reason {
        ParserErrorReason::Incomplete { cause, kind } => match kind {
            IncompleteKind::UnclosedBrace => {
                let message = "unclosed brace found here";
                let location = ErrorLocation::from_source_and_offset(source, cause.start());
                vec![(Context::Note(message.to_string()), Some(location))]
            }
            IncompleteKind::UnclosedBracket => {
                let message = "unclosed bracket found here";
                let location = ErrorLocation::from_source_and_offset(source, cause.start());
                vec![(Context::Note(message.to_string()), Some(location))]
            }
            IncompleteKind::UnclosedParen => {
                let message = "unclosed parenthesis found here";
                let location = ErrorLocation::from_source_and_offset(source, cause.start());
                vec![(Context::Note(message.to_string()), Some(location))]
            }
            _ => vec![],
        },

        ParserErrorReason::TokenError(kind) => match kind {
            TokenErrorKind::Incomplete(kind) => match kind {
                TokenIncompleteKind::UnclosedNote {
                    delimiter_start_offset,
                    delimiter_length,
                } => {
                    let message = "unclosed note found here";
                    let location = ErrorLocation::from_source_and_span(
                        source,
                        *delimiter_start_offset,
                        *delimiter_length,
                    );
                    vec![(Context::Note(message.to_string()), Some(location))]
                }
                TokenIncompleteKind::UnclosedString { open_quote_offset } => {
                    let message = "unclosed string found here";
                    let location =
                        ErrorLocation::from_source_and_offset(source, *open_quote_offset);
                    vec![(Context::Note(message.to_string()), Some(location))]
                }
                _ => vec![],
            },
            _ => vec![],
        },

        _ => vec![],
    }
}

fn invalid_number_literal(
    reason: &ParserErrorReason,
    source: &str,
) -> Vec<(Context, Option<ErrorLocation>)> {
    match reason {
        ParserErrorReason::TokenError(kind) => match kind {
            TokenErrorKind::Incomplete(kind) => match kind {
                TokenIncompleteKind::InvalidDecimalPart {
                    decimal_point_offset,
                } => {
                    let message = "because of `.` here";
                    let location =
                        ErrorLocation::from_source_and_offset(source, *decimal_point_offset);
                    vec![(Context::Note(message.to_string()), Some(location))]
                }
                TokenIncompleteKind::InvalidExponentPart { e_offset } => {
                    let message = "because of `e` here";
                    let location = ErrorLocation::from_source_and_offset(source, *e_offset);
                    vec![(Context::Note(message.to_string()), Some(location))]
                }
                _ => vec![],
            },
            _ => vec![],
        },
        _ => vec![],
    }
}

mod parsers {
    use nom::{
        AsChar, IResult, Input, Parser,
        bytes::complete::take_while,
        character::complete::{char as nom_char, satisfy},
        error::ParseError,
        multi::many0,
    };

    fn alphanumeric(input: &str) -> IResult<&str, ()> {
        let (input, _) = satisfy(|c| c.is_ascii_alphanumeric()).parse(input)?;
        Ok((input, ()))
    }

    fn char<I, E>(c: char) -> impl Parser<I, Output = (), Error = E>
    where
        I: Input,
        I::Item: AsChar,
        E: ParseError<I>,
    {
        nom_char::<_, E>(c).map(|_| ())
    }

    fn ident(input: &str) -> IResult<&str, ()> {
        let underscore = char('_');

        let (input, _) = alphanumeric.parse(input)?;
        let (input, _) = many0(underscore.or(alphanumeric)).parse(input)?;

        Ok((input, ()))
    }

    fn whitespace(input: &str) -> IResult<&str, ()> {
        let (input, _) = take_while(char::is_whitespace).parse(input)?;

        Ok((input, ()))
    }

    pub fn ident_and_equals(input: &str) -> IResult<&str, ()> {
        let (input, _) = ident(input)?;
        let (input, _) = whitespace(input)?;
        let (input, _) = char('=').parse(input)?;

        Ok((input, ()))
    }
}
