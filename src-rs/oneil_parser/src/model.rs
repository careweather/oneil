//! Parser for model definitions in an Oneil program.

use nom::{
    Parser as _,
    bytes::complete::take_while,
    combinator::{cut, eof, opt, value},
};

use oneil_ast::{
    declaration::Decl,
    model::{Model, Section},
};

use crate::{
    declaration::parse as parse_decl,
    error::{
        ErrorHandlingParser, ExpectKind, ParserError, ParserErrorKind,
        partial::ErrorsWithPartialResult,
    },
    note::parse as parse_note,
    token::{Token, keyword::section, naming::label, structure::end_of_line},
    util::{Result, Span},
};

/// Parses a model definition, consuming the complete input
///
/// This function **fails if the complete input is not consumed**.
pub fn parse_complete(input: Span) -> Result<Model, ErrorsWithPartialResult<Model, ParserError>> {
    let (rest, model) = model(input)?;
    let result = eof(rest);

    match result {
        Ok((rest, _)) => Ok((rest, model)),
        Err(nom::Err::Error(e)) => Err(nom::Err::Failure(ErrorsWithPartialResult::new(
            model,
            vec![e],
        ))),
        _ => unreachable!(),
    }
}

/// Parses a model definition
fn model(input: Span) -> Result<Model, ErrorsWithPartialResult<Model, ParserError>> {
    let (rest, _) = opt(end_of_line).convert_errors().parse(input)?;
    let (rest, note) = opt(parse_note).convert_errors().parse(rest)?;
    let (rest, decls, decl_errors) = parse_decls(rest);
    let (rest, sections, section_errors) = parse_sections(rest);

    let errors = [decl_errors, section_errors].concat();

    if errors.is_empty() {
        Ok((
            rest,
            Model {
                note,
                decls,
                sections,
            },
        ))
    } else {
        Err(nom::Err::Failure(ErrorsWithPartialResult::new(
            Model {
                note,
                decls,
                sections,
            },
            errors,
        )))
    }
}

/// Attempts to parse declarations
///
/// If it fails to parse a declaration, it attempts to recover and continue
/// parsing. This allows for multiple syntax errors to be found in the model.
///
/// In addition, because it returns partial results, the results may be used
/// in order to determine other partial information, such as the associated
/// units of the declarations that were successfully parsed.
fn parse_decls(input: Span) -> (Span, Vec<Decl>, Vec<ParserError>) {
    fn parse_decls_recur<'a>(
        input: Span<'a>,
        mut acc_decls: Vec<Decl>,
        mut acc_errors: Vec<ParserError>,
        last_was_error: bool,
    ) -> (Span<'a>, Vec<Decl>, Vec<ParserError>) {
        let result = parse_decl(input);

        match result {
            Ok((rest, decl)) => {
                acc_decls.push(decl);
                parse_decls_recur(rest, acc_decls, acc_errors, false)
            }

            Err(nom::Err::Error(e) | nom::Err::Failure(e)) => {
                // Check if a section or the end of the file is next
                // If it is, return the accumulated declarations and errors
                let end_of_file = value((), take_while(char::is_whitespace).and(eof));
                let section = value((), section);
                if let Ok(_) = section.or(end_of_file).parse(input) {
                    return (input, acc_decls, acc_errors);
                }

                // We don't want to add the error if the current line could be a
                // part of a previous faulty declaration, such as in the case of
                // a piecewise function. ExpectDecl is the only possible Error,
                // and it isn't a possible Failure, so we can use it to check
                // if we were simply unable to find a declaration, rather than
                // if we found a declaration, but it was invalid.
                let is_possible_part_of_previous_decl =
                    last_was_error && e.kind == ParserErrorKind::Expect(ExpectKind::Decl);

                if !is_possible_part_of_previous_decl {
                    acc_errors.push(e);
                }

                // All declarations must be terminated by an end of line, so we
                // assume that the declaration parsing error is for a declaration
                // that ends at the end of the line
                let next_line = skip_to_next_line(input);

                parse_decls_recur(next_line, acc_decls, acc_errors, true)
            }
            Err(nom::Err::Incomplete(_needed)) => (input, acc_decls, acc_errors),
        }
    }

    parse_decls_recur(input, vec![], vec![], false)
}

/// Parses the sections of a model
fn parse_sections(input: Span) -> (Span, Vec<Section>, Vec<ParserError>) {
    fn parse_sections_recur<'a>(
        input: Span<'a>,
        mut acc_sections: Vec<Section>,
        mut acc_errors: Vec<ParserError>,
    ) -> (Span<'a>, Vec<Section>, Vec<ParserError>) {
        let section_result = parse_section(input);

        match section_result {
            Some((rest, section, errors)) => {
                acc_sections.push(section);
                acc_errors.extend(errors);
                parse_sections_recur(rest, acc_sections, acc_errors)
            }
            None => (input, acc_sections, acc_errors),
        }
    }

    parse_sections_recur(input, vec![], vec![])
}

/// Parses a section within a model
///
/// If there is no section header, this function returns `None`, indicating that
/// no section was found.
///
/// Otherwise, this function returns a tuple containing the section and the
/// errors that occurred while parsing the section, if any.
fn parse_section(input: Span) -> Option<(Span, Section, Vec<ParserError>)> {
    let section_header_result = parse_section_header(input);

    let (rest, label, mut errors) = match section_header_result {
        Ok((rest, label)) => (rest, Some(label), vec![]),
        Err(nom::Err::Error(_e)) => {
            // No section header was found, so we return None
            return None;
        }
        Err(nom::Err::Failure(e)) => {
            // There was a problem with the section header, so we keep the error and skip to the next line
            let rest = skip_to_next_line(input);
            (rest, None, vec![e])
        }
        Err(nom::Err::Incomplete(_needed)) => (input, None, vec![]),
    };

    let (rest, note) = opt(parse_note)
        .parse(rest)
        .expect("should always parse because its optional");

    let (rest, decls, decl_errors) = parse_decls(rest);
    errors.extend(decl_errors);

    let label = label.map_or("<FAILED TO PARSE SECTION LABEL>".to_string(), |label| {
        label.lexeme().to_string()
    });

    Some((rest, Section { label, note, decls }, errors))
}

fn parse_section_header(input: Span) -> Result<Token, ParserError> {
    let (rest, section_span) = section.convert_errors().parse(input)?;

    let (rest, label) = cut(label)
        .map_failure(ParserError::section_missing_label(section_span))
        .parse(rest)?;

    let (rest, _) = cut(end_of_line)
        .map_failure(ParserError::section_missing_end_of_line(section_span))
        .parse(rest)?;

    Ok((rest, label))
}

/// Attempts to recover from a parsing error by skipping to the next line
fn skip_to_next_line(input: Span) -> Span {
    let (rest, _) = take_while::<_, _, nom::error::Error<_>>(|c| c != '\n')
        .parse(input)
        .expect("should never fail");

    let (rest, _) = end_of_line
        .parse(rest)
        .expect("should always parse either a line break or EOF");

    rest
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;
    use oneil_ast::declaration::{Decl, Import};

    #[test]
    fn test_empty_model() {
        let input = Span::new_extra("", Config::default());
        let (rest, model) = parse_complete(input).unwrap();
        assert!(model.note.is_none());
        assert!(model.decls.is_empty());
        assert!(model.sections.is_empty());
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_model_with_note() {
        let input = Span::new_extra("~ This is a note\n", Config::default());
        let (rest, model) = parse_complete(input).unwrap();
        assert!(model.note.is_some());
        assert!(model.decls.is_empty());
        assert!(model.sections.is_empty());
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_model_with_import() {
        let input = Span::new_extra("import foo\n", Config::default());
        let (rest, model) = parse_complete(input).unwrap();
        assert!(model.note.is_none());
        assert_eq!(model.decls.len(), 1);
        match &model.decls[0] {
            Decl::Import(Import { path }) => assert_eq!(path, "foo"),
            _ => panic!("Expected import declaration"),
        }
        assert!(model.sections.is_empty());
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_model_with_section() {
        let input = Span::new_extra("section foo\nimport bar\n", Config::default());
        let (rest, model) = parse_complete(input).unwrap();
        assert!(model.note.is_none());
        assert!(model.decls.is_empty());
        assert_eq!(model.sections.len(), 1);
        let section = &model.sections[0];
        assert_eq!(section.label, "foo");
        assert_eq!(section.decls.len(), 1);
        match &section.decls[0] {
            Decl::Import(Import { path }) => assert_eq!(path, "bar"),
            _ => panic!("Expected import declaration"),
        }
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_model_with_multiple_sections() {
        let input = Span::new_extra(
            "section foo\nimport bar\nsection baz\nimport qux\n",
            Config::default(),
        );
        let (rest, model) = parse_complete(input).unwrap();
        assert!(model.note.is_none());
        assert!(model.decls.is_empty());
        assert_eq!(model.sections.len(), 2);

        let section1 = &model.sections[0];
        assert_eq!(section1.label, "foo");
        assert_eq!(section1.decls.len(), 1);
        match &section1.decls[0] {
            Decl::Import(Import { path }) => assert_eq!(path, "bar"),
            _ => panic!("Expected import declaration"),
        }

        let section2 = &model.sections[1];
        assert_eq!(section2.label, "baz");
        assert_eq!(section2.decls.len(), 1);
        match &section2.decls[0] {
            Decl::Import(Import { path }) => assert_eq!(path, "qux"),
            _ => panic!("Expected import declaration"),
        }

        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_parse_complete_empty_model_success() {
        let input = Span::new_extra("\n", Config::default());
        let (rest, model) = parse_complete(input).unwrap();
        assert!(model.note.is_none());
        assert!(model.decls.is_empty());
        assert!(model.sections.is_empty());
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_parse_complete_with_declarations_success() {
        let input = Span::new_extra("import foo\nimport bar\n", Config::default());
        let (rest, model) = parse_complete(input).unwrap();
        assert_eq!(model.decls.len(), 2);
        match &model.decls[0] {
            Decl::Import(Import { path }) => assert_eq!(path, "foo"),
            _ => panic!("Expected import declaration"),
        }
        match &model.decls[1] {
            Decl::Import(Import { path }) => assert_eq!(path, "bar"),
            _ => panic!("Expected import declaration"),
        }
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_parse_complete_with_remaining_input() {
        let input = Span::new_extra("import foo\n<rest>", Config::default());
        let result = parse_complete(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_model() {
        let input = Span::new_extra("", Config::default());
        let (rest, model) = parse_complete(input).unwrap();
        assert!(model.note.is_none());
        assert!(model.decls.is_empty());
        assert!(model.sections.is_empty());
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_parse_model_with_parameters() {
        let input = Span::new_extra(
            "1st parameter: x = 1\n2nd parameter: y = 2\n",
            Config::default(),
        );
        let (rest, model) = parse_complete(input).unwrap();
        assert!(model.note.is_none());
        assert_eq!(model.decls.len(), 2);
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_parse_model_with_section_and_declarations() {
        let input = Span::new_extra(
            "X: x = 1 + 2\nsection My Section\nimport foo\nimport bar\nY: y = 3 * 4",
            Config::default(),
        );
        let (rest, model) = parse_complete(input).unwrap();
        assert!(model.note.is_none());
        assert_eq!(model.decls.len(), 1);
        assert_eq!(model.sections.len(), 1);
        let section = &model.sections[0];
        assert_eq!(section.label, "My Section");
        assert_eq!(section.decls.len(), 3);
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_parse_model_failure_with_partial_result() {
        let input = Span::new_extra(
            "\
            use foo as bar

            from foo use as baz # missing `use` part

            X: x = 1 + # incomplete

            section My Section

            use foo as bar

            import # missing import identifier

            Y: y = 3 * 4 : # missing unit
            ",
            Config::default(),
        );

        let result = parse_complete(input);
        assert!(result.is_err());

        match result {
            Err(nom::Err::Failure(e)) => {
                let model = e.partial_result;
                let errors = e.errors;

                assert_eq!(model.decls.len(), 1);
                assert_eq!(model.sections.len(), 1);
                assert_eq!(model.sections[0].label, "My Section");
                assert_eq!(model.sections[0].decls.len(), 1);

                assert_eq!(errors.len(), 4);
            }
            _ => panic!("Expected an error with incomplete input"),
        }
    }
}
