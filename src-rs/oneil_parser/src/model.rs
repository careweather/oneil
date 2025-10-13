//! Parser for model definitions in an Oneil program.
//!
//! The parser uses error recovery to continue parsing even when individual
//! declarations or sections fail, allowing multiple syntax errors to be
//! reported in a single pass.

use std::result::Result as StdResult;

use nom::{
    Parser as _,
    bytes::complete::take_while,
    combinator::{eof, opt, value},
};
use oneil_ast::{
    AstSpan, DeclNode, Label, Model, ModelNode, Node, Section, SectionHeader, SectionHeaderNode,
    SectionNode,
};

use crate::{
    declaration::parse as parse_decl,
    error::{
        ErrorHandlingParser, ParserError,
        partial::ErrorsWithPartialResult,
        reason::{ExpectKind, ParserErrorReason},
    },
    note::parse as parse_note,
    token::{keyword::section, naming::label, structure::end_of_line},
    util::{InputSpan, Result},
};

/// Parses a model definition, consuming the complete input
///
/// This function **fails if the complete input is not consumed**.
pub fn parse_complete(
    input: InputSpan<'_>,
) -> Result<'_, ModelNode, ErrorsWithPartialResult<Box<Model>, ParserError>> {
    let (rest, model) = model(input)?;
    let result = eof(rest);

    match result {
        Ok((rest, _)) => Ok((rest, model)),
        Err(nom::Err::Error(e)) => Err(nom::Err::Failure(ErrorsWithPartialResult::new(
            Box::new(model.take_value()),
            vec![e],
        ))),
        _ => unreachable!(),
    }
}

/// Parses a complete model definition with error recovery.
///
/// The function uses error recovery to continue parsing even when individual
/// declarations or sections fail, allowing multiple syntax errors to be
/// reported in a single pass.
fn model(
    input: InputSpan<'_>,
) -> Result<'_, ModelNode, ErrorsWithPartialResult<Box<Model>, ParserError>> {
    let (rest, _) = opt(end_of_line).convert_errors().parse(input)?;
    let (rest, note) = opt(parse_note).convert_errors().parse(rest)?;
    let (rest, mut decls, decl_errors) = parse_decls(rest);
    let (rest, sections, decls_without_section, section_errors) = parse_sections(rest);

    // for any decls where the section header parsing failed, add them to the top-level decls
    decls.extend(decls_without_section);

    let errors = [decl_errors, section_errors].concat();

    if errors.is_empty() {
        // assume that the model spans the entire file
        let model_span = AstSpan::new(0, input.len(), 0);
        let model_node = Node::new(&model_span, Model::new(note, decls, sections));

        Ok((rest, model_node))
    } else {
        let model = Box::new(Model::new(note, decls, sections));
        Err(nom::Err::Failure(ErrorsWithPartialResult::new(
            model, errors,
        )))
    }
}

/// Attempts to parse declarations with error recovery
///
/// The function handles consecutive errors by avoiding duplicate error
/// reporting for lines that might be continuations of previous failed
/// declarations (e.g., multi-line piecewise functions).
fn parse_decls(input: InputSpan<'_>) -> (InputSpan<'_>, Vec<DeclNode>, Vec<ParserError>) {
    fn parse_decls_recur(
        input: InputSpan<'_>,
        mut acc_decls: Vec<DeclNode>,
        mut acc_errors: Vec<ParserError>,
        last_was_error: bool,
    ) -> (InputSpan<'_>, Vec<DeclNode>, Vec<ParserError>) {
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
                if section.or(end_of_file).parse(input).is_ok() {
                    return (input, acc_decls, acc_errors);
                }

                // We don't want to add the error if the current line could be a
                // part of a previous faulty declaration, such as in the case of
                // a piecewise function. ExpectDecl is the only possible Error,
                // and it isn't a possible Failure, so we can use it to check
                // if we were simply unable to find a declaration, rather than
                // if we found a declaration, but it was invalid.
                let is_possible_part_of_previous_decl =
                    last_was_error && e.reason == ParserErrorReason::Expect(ExpectKind::Decl);

                if !is_possible_part_of_previous_decl {
                    acc_errors.push(e);
                }

                // All declarations must be terminated by an end of line, so we
                // assume that the declaration parsing error is for a declaration
                // that ends at the end of the line
                let next_line = skip_to_next_line_with_content(input);

                parse_decls_recur(next_line, acc_decls, acc_errors, true)
            }
            Err(nom::Err::Incomplete(_needed)) => (input, acc_decls, acc_errors),
        }
    }

    parse_decls_recur(input, vec![], vec![], false)
}

/// Parses the sections of a model with error recovery
fn parse_sections(
    input: InputSpan<'_>,
) -> (
    InputSpan<'_>,
    Vec<SectionNode>,
    Vec<DeclNode>,
    Vec<ParserError>,
) {
    fn parse_sections_recur(
        input: InputSpan<'_>,
        mut acc_sections: Vec<SectionNode>,
        mut acc_decls: Vec<DeclNode>,
        mut acc_errors: Vec<ParserError>,
    ) -> (
        InputSpan<'_>,
        Vec<SectionNode>,
        Vec<DeclNode>,
        Vec<ParserError>,
    ) {
        let section_result = parse_section(input);

        match section_result {
            Some((rest, section_result, errors)) => {
                match section_result {
                    Ok(section) => {
                        // if the section was parsed successfully, add it to the accumulator
                        acc_sections.push(section);
                    }
                    Err(decls) => {
                        // if the section was not parsed successfully, add the decls to the top-level decls
                        acc_decls.extend(decls);
                    }
                }

                acc_errors.extend(errors);
                parse_sections_recur(rest, acc_sections, acc_decls, acc_errors)
            }
            None => (input, acc_sections, acc_decls, acc_errors),
        }
    }

    parse_sections_recur(input, vec![], vec![], vec![])
}

type SectionResult = StdResult<SectionNode, Vec<DeclNode>>;

type SectionErrors = Vec<ParserError>;

/// Parses a section within a model
fn parse_section(input: InputSpan<'_>) -> Option<(InputSpan<'_>, SectionResult, SectionErrors)> {
    let section_header_result = parse_section_header(input);

    let (rest, header, mut errors) = match section_header_result {
        Ok((rest, header)) => (rest, Some(header), vec![]),
        Err(nom::Err::Error(_e)) => {
            // No section header was found, so we return None
            return None;
        }
        Err(nom::Err::Failure(e)) => {
            // There was a problem with the section header, so we keep the error and skip to the next line
            let rest = skip_to_next_line_with_content(input);
            (rest, None, vec![e])
        }
        Err(nom::Err::Incomplete(_needed)) => (input, None, vec![]),
    };

    let (rest, note) = opt(parse_note)
        .parse(rest)
        .expect("should always parse because its optional");

    let (rest, decls, decl_errors) = parse_decls(rest);
    errors.extend(decl_errors);

    match header {
        Some(header) => {
            let span_start = &header;
            let span_end = match (&note, decls.last()) {
                (_, Some(decl)) => AstSpan::from(decl),
                (Some(note), _) => AstSpan::from(note),
                (_, _) => AstSpan::from(&header),
            };

            let span = AstSpan::calc_span(&span_start, &span_end);

            let section_node = Node::new(&span, Section::new(header, note, decls));

            Some((rest, Ok(section_node), errors))
        }
        // if there was a problem with the section header, return the decls parsed so that
        // they can be merged with the top-level decls
        None => Some((rest, Err(decls), errors)),
    }
}

/// Parses a section header with its label
fn parse_section_header(input: InputSpan<'_>) -> Result<'_, SectionHeaderNode, ParserError> {
    let (rest, section_span) = section.convert_errors().parse(input)?;

    let (rest, label) = label
        .or_fail_with(ParserError::section_missing_label(&section_span))
        .parse(rest)?;
    let label_value = Label::new(label.lexeme().to_string());
    let label_node = Node::new(&label, label_value);

    let (rest, end_of_line_token) = end_of_line
        .or_fail_with(ParserError::section_missing_end_of_line(&label))
        .parse(rest)?;

    let span = AstSpan::calc_span_with_whitespace(&section_span, &label, &end_of_line_token);
    let header_node = Node::new(&span, SectionHeader::new(label_node));

    Ok((rest, header_node))
}

/// Attempts to recover from a parsing error by skipping to the next line
///
/// This function is used for error recovery when parsing declarations or
/// section headers. It skips all characters until it finds a newline or
/// end of file, then consumes the newline character itself.
///
/// It also optionally skips a note that follows the line break.
fn skip_to_next_line_with_content(input: InputSpan<'_>) -> InputSpan<'_> {
    let (rest, _) = take_while::<_, _, nom::error::Error<_>>(|c| c != '\n')
        .parse(input)
        .expect("should never fail");

    let (rest, _) = end_of_line
        .parse(rest)
        .expect("should always parse either a line break or EOF");

    let (rest, _) = opt(parse_note)
        .parse(rest)
        .expect("should always parse because its optional");

    rest
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;
    use oneil_ast::Decl;

    #[test]
    fn empty_model() {
        let input = InputSpan::new_extra("", Config::default());
        let (rest, model) = parse_complete(input).expect("should parse empty model");
        assert!(model.note().is_none());
        assert!(model.decls().is_empty());
        assert!(model.sections().is_empty());
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn model_with_note() {
        let input = InputSpan::new_extra("~ This is a note\n", Config::default());
        let (rest, model) = parse_complete(input).expect("should parse model with note");
        assert!(model.note().is_some());
        assert!(model.decls().is_empty());
        assert!(model.sections().is_empty());
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn model_with_import() {
        let input = InputSpan::new_extra("import foo\n", Config::default());
        let (rest, model) = parse_complete(input).expect("should parse model with import");
        assert!(model.note().is_none());
        assert_eq!(model.decls().len(), 1);

        let Decl::Import(import_node) = &model.decls()[0].node_value() else {
            panic!("Expected import declaration");
        };
        assert_eq!(import_node.path().node_value(), &"foo");

        assert!(model.sections().is_empty());
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn use_without_as() {
        let input = InputSpan::new_extra("use foo\n", Config::default());
        let (rest, model) = parse_complete(input).expect("should parse model with use declaration");
        assert!(model.note().is_none());
        assert_eq!(model.decls().len(), 1);

        let Decl::UseModel(use_model_node) = &model.decls()[0].node_value() else {
            panic!("Expected use declaration");
        };

        let use_model_info = use_model_node.model_info();
        assert_eq!(use_model_info.top_component().as_str(), "foo");
        assert_eq!(use_model_info.subcomponents().len(), 0);
        assert_eq!(use_model_info.get_alias().as_str(), "foo");

        assert!(model.sections().is_empty());
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn model_with_section() {
        let input = InputSpan::new_extra("section foo\nimport bar\n", Config::default());
        let (rest, model) = parse_complete(input).expect("should parse model with section");
        assert!(model.note().is_none());
        assert!(model.decls().is_empty());
        assert_eq!(model.sections().len(), 1);
        let section = &model.sections()[0];
        assert_eq!(section.header().label().as_str(), "foo");
        assert_eq!(section.decls().len(), 1);
        let Decl::Import(import_node) = &section.decls()[0].node_value() else {
            panic!("Expected import declaration");
        };
        assert_eq!(import_node.path().node_value(), "bar");
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn model_with_multiple_sections() {
        let input = InputSpan::new_extra(
            "section foo\nimport bar\nsection baz\nimport qux\n",
            Config::default(),
        );
        let (rest, model) =
            parse_complete(input).expect("should parse model with multiple sections");
        assert!(model.note().is_none());
        assert!(model.decls().is_empty());
        assert_eq!(model.sections().len(), 2);

        let section1 = &model.sections()[0];
        assert_eq!(section1.header().label().as_str(), "foo");
        assert_eq!(section1.decls().len(), 1);
        let Decl::Import(import_node) = &section1.decls()[0].node_value() else {
            panic!("Expected import declaration");
        };
        assert_eq!(import_node.path().node_value(), "bar");

        let section2 = &model.sections()[1];
        assert_eq!(section2.header().label().as_str(), "baz");
        assert_eq!(section2.decls().len(), 1);
        let Decl::Import(import_node) = &section2.decls()[0].node_value() else {
            panic!("Expected import declaration");
        };
        assert_eq!(import_node.path().node_value(), "qux");

        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn parse_complete_empty_model_success() {
        let input = InputSpan::new_extra("\n", Config::default());
        let (rest, model) = parse_complete(input).expect("should parse empty model");
        assert!(model.note().is_none());
        assert!(model.decls().is_empty());
        assert!(model.sections().is_empty());
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn parse_complete_with_declarations_success() {
        let input = InputSpan::new_extra("import foo\nimport bar\n", Config::default());
        let (rest, model) = parse_complete(input).expect("should parse model with declarations");
        assert_eq!(model.decls().len(), 2);
        let Decl::Import(import_node) = &model.decls()[0].node_value() else {
            panic!("Expected import declaration");
        };
        assert_eq!(import_node.path().node_value(), "foo");
        let Decl::Import(import_node) = &model.decls()[1].node_value() else {
            panic!("Expected import declaration");
        };
        assert_eq!(import_node.path().node_value(), "bar");
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    #[expect(
        clippy::assertions_on_result_states,
        reason = "we don't care about the result, just that it's an error"
    )]
    fn parse_complete_with_remaining_input() {
        let input = InputSpan::new_extra("import foo\n<rest>", Config::default());
        let result = parse_complete(input);
        assert!(result.is_err());
    }

    #[test]
    fn parse_empty_model() {
        let input = InputSpan::new_extra("", Config::default());
        let (rest, model) = parse_complete(input).expect("should parse empty model");
        assert!(model.note().is_none());
        assert!(model.decls().is_empty());
        assert!(model.sections().is_empty());
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn parse_model_with_parameters() {
        let input = InputSpan::new_extra(
            "1st parameter: x = 1\n2nd parameter: y = 2\n",
            Config::default(),
        );
        let (rest, model) = parse_complete(input).expect("should parse model with parameters");
        assert!(model.note().is_none());
        assert_eq!(model.decls().len(), 2);
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn parse_model_with_section_and_declarations() {
        let input = InputSpan::new_extra(
            "X: x = 1 + 2\nsection My Section\nimport foo\nimport bar\nY: y = 3 * 4",
            Config::default(),
        );
        let (rest, model) =
            parse_complete(input).expect("should parse model with section and declarations");
        assert!(model.note().is_none());
        assert_eq!(model.decls().len(), 1);
        assert_eq!(model.sections().len(), 1);
        let section = &model.sections()[0];
        assert_eq!(section.header().label().as_str(), "My Section");
        assert_eq!(section.decls().len(), 3);
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn parse_model_failure_with_partial_result() {
        let input = InputSpan::new_extra(
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

        let Err(nom::Err::Failure(e)) = result else {
            panic!("Expected error for incomplete input");
        };

        let model = e.partial_result;
        let errors = e.errors;

        assert_eq!(model.decls().len(), 1);
        assert_eq!(model.sections().len(), 1);
        assert_eq!(model.sections()[0].header().label().as_str(), "My Section");
        assert_eq!(model.sections()[0].decls().len(), 1);

        assert_eq!(errors.len(), 4);
    }

    mod general_error {
        use crate::error::reason::{ExpectKind, ParserErrorReason};

        use super::*;

        #[test]
        fn parse_complete_with_remaining_input() {
            let input = InputSpan::new_extra("import foo\nrest", Config::default());
            let result = parse_complete(input);

            let Err(nom::Err::Failure(e)) = result else {
                panic!("Expected error for remaining input");
            };

            let model = e.partial_result;
            let errors = e.errors;

            assert_eq!(model.decls().len(), 1);
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].error_offset, 11);
            assert_eq!(
                errors[0].reason,
                ParserErrorReason::Expect(ExpectKind::Decl)
            );
        }
    }

    mod section_error {
        use crate::error::reason::{IncompleteKind, ParserErrorReason, SectionKind};

        use super::*;

        #[test]
        fn section_missing_label() {
            let input = InputSpan::new_extra("section\nimport foo\n", Config::default());
            let result = parse_complete(input);
            let Err(nom::Err::Failure(e)) = result else {
                panic!("Expected error for missing section label");
            };

            let model = e.partial_result;
            let errors = e.errors;

            assert_eq!(model.decls().len(), 1);
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].error_offset, 7);
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Section(SectionKind::MissingLabel),
                cause,
            } = errors[0].reason
            else {
                panic!("Unexpected reason {:?}", errors[0].reason);
            };

            assert_eq!(cause, AstSpan::new(0, 7, 0));
        }

        #[test]
        fn section_missing_end_of_line() {
            let input = InputSpan::new_extra("section foo :\n import foo", Config::default());
            let result = parse_complete(input);
            let Err(nom::Err::Failure(e)) = result else {
                panic!("Expected error for missing section label, got {result:?}");
            };

            let model = e.partial_result;
            let errors = e.errors;

            assert_eq!(model.decls().len(), 1);
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].error_offset, 12);
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Section(SectionKind::MissingEndOfLine),
                cause,
            } = errors[0].reason
            else {
                panic!("Unexpected reason {:?}", errors[0].reason);
            };

            assert_eq!(cause, AstSpan::new(8, 3, 1));
        }

        #[test]
        fn section_with_invalid_declaration() {
            let input = InputSpan::new_extra("section foo\nimport\n", Config::default());
            let result = parse_complete(input);
            let Err(nom::Err::Failure(e)) = result else {
                panic!("Expected error for invalid declaration in section");
            };

            let model = e.partial_result;
            let errors = e.errors;

            assert_eq!(model.sections().len(), 1);
            assert_eq!(model.sections()[0].decls().len(), 0);
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].error_offset, 18);
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Decl(_),
                cause,
            } = errors[0].reason
            else {
                panic!("Unexpected reason {:?}", errors[0].reason);
            };

            assert_eq!(cause, AstSpan::new(12, 6, 0));
        }
    }

    mod declaration_error {
        use crate::error::reason::{IncompleteKind, ParserErrorReason};

        use super::*;

        #[test]
        fn import_missing_path() {
            let input = InputSpan::new_extra("import\n", Config::default());
            let result = parse_complete(input);
            let Err(nom::Err::Failure(e)) = result else {
                panic!("Expected error for missing import path");
            };

            let model = e.partial_result;
            let errors = e.errors;

            assert_eq!(model.decls().len(), 0);
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].error_offset, 6);
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Decl(_),
                cause,
            } = errors[0].reason
            else {
                panic!("Unexpected reason {:?}", errors[0].reason);
            };

            assert_eq!(cause, AstSpan::new(0, 6, 0));
        }

        #[test]
        fn parameter_missing_equals() {
            let input = InputSpan::new_extra("X: x\n", Config::default());
            let result = parse_complete(input);
            let Err(nom::Err::Failure(e)) = result else {
                panic!("Expected error for missing equals in parameter");
            };

            let model = e.partial_result;
            let errors = e.errors;

            assert_eq!(model.decls().len(), 0);
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].error_offset, 4);
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Parameter(_),
                cause,
            } = errors[0].reason
            else {
                panic!("Unexpected reason {:?}", errors[0].reason);
            };

            assert_eq!(cause, AstSpan::new(3, 1, 0));
        }

        #[test]
        fn parameter_missing_value() {
            let input = InputSpan::new_extra("X: x =\n", Config::default());
            let result = parse_complete(input);
            let Err(nom::Err::Failure(e)) = result else {
                panic!("Expected error for missing value in parameter");
            };

            let model = e.partial_result;
            let errors = e.errors;

            assert_eq!(model.decls().len(), 0);
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].error_offset, 6);
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Parameter(_),
                cause,
            } = errors[0].reason
            else {
                panic!("Unexpected reason {:?}", errors[0].reason);
            };

            assert_eq!(cause, AstSpan::new(5, 1, 0));
        }

        #[test]
        fn missing_colon() {
            let input = InputSpan::new_extra("test x > 0\n", Config::default());
            let result = parse_complete(input);
            let Err(nom::Err::Failure(e)) = result else {
                panic!("Expected error for missing colon in test");
            };

            let model = e.partial_result;
            let errors = e.errors;

            assert_eq!(model.decls().len(), 0);
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].error_offset, 5);
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Test(_),
                cause,
            } = errors[0].reason
            else {
                panic!("Unexpected reason {:?}", errors[0].reason);
            };

            assert_eq!(cause, AstSpan::new(0, 4, 1));
        }
    }

    mod note_error {
        use crate::error::reason::ParserErrorReason;

        use super::*;

        #[test]
        fn unterminated_note() {
            let input =
                InputSpan::new_extra("~~~\nThis is an unterminated note", Config::default());
            let result = parse_complete(input);
            let Err(nom::Err::Failure(e)) = result else {
                panic!("Expected error for unterminated note");
            };

            let model = e.partial_result;
            let errors = e.errors;

            assert_eq!(model.decls().len(), 0);
            assert_eq!(errors.len(), 1);
            let ParserErrorReason::TokenError(_) = errors[0].reason else {
                panic!("Unexpected reason {:?}", errors[0].reason);
            };
        }
    }

    mod recovery_error {
        use crate::error::reason::{IncompleteKind, ParserErrorReason};

        use super::*;

        #[test]
        fn multiple_declaration_errors() {
            let input = InputSpan::new_extra("import\nuse\nX: x\n", Config::default());
            let result = parse_complete(input);
            let Err(nom::Err::Failure(e)) = result else {
                panic!("Expected error for multiple declaration errors");
            };

            let model = e.partial_result;
            let errors = e.errors;

            assert_eq!(model.decls().len(), 0);
            assert_eq!(errors.len(), 3);

            // All errors should be declaration-related
            for error in &errors {
                let ParserErrorReason::Incomplete {
                    kind: IncompleteKind::Decl(_) | IncompleteKind::Parameter(_),
                    ..
                } = error.reason
                else {
                    panic!("Expected declaration error, got {:?}", error.reason);
                };
            }
        }

        #[test]
        fn section_with_multiple_errors() {
            let input = InputSpan::new_extra("section foo\nimport\nuse\nX: x\n", Config::default());
            let result = parse_complete(input);
            let Err(nom::Err::Failure(e)) = result else {
                panic!("Expected error for section with multiple errors");
            };

            let model = e.partial_result;
            let errors = e.errors;

            assert_eq!(model.sections().len(), 1);
            assert_eq!(model.sections()[0].decls().len(), 0);
            assert_eq!(errors.len(), 3);
        }

        #[test]
        fn mixed_valid_and_invalid_declarations() {
            let input = InputSpan::new_extra(
                "import valid\nimport\nuse foo as bar\nuse invalid.\n",
                Config::default(),
            );
            let result = parse_complete(input);
            let Err(nom::Err::Failure(e)) = result else {
                panic!("Expected error for mixed valid and invalid declarations");
            };

            let model = e.partial_result;
            let errors = e.errors;

            // Should have successfully parsed some declarations
            assert_eq!(model.decls().len(), 2);
            assert_eq!(errors.len(), 2);

            // Check that the valid declarations were parsed
            let Decl::Import(import_node) = &model.decls()[0].node_value() else {
                panic!("Expected import declaration");
            };
            assert_eq!(import_node.path().node_value(), "valid");
            let Decl::UseModel(use_node) = &model.decls()[1].node_value() else {
                panic!("Expected use model declaration");
            };
            let alias = use_node.model_info().get_alias();
            assert_eq!(alias.as_str(), "bar");
        }
    }
}
