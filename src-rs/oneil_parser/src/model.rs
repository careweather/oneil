//! Parser for model definitions in an Oneil program.
//!
//! This module provides parsing functionality for complete Oneil model definitions.
//! A model consists of:
//!
//! 1. **Optional leading whitespace and newlines** - Ignored during parsing
//! 2. **Optional note** - A comment starting with `~` at the beginning of the model
//! 3. **Top-level declarations** - Import, from, use, parameter, and test declarations
//! 4. **Sections** - Named groups of declarations with the format `section <label>`
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
    Span as AstSpan,
    declaration::DeclNode,
    model::{Model, ModelNode, Section, SectionHeader, SectionHeaderNode, SectionNode},
    naming::Label,
    node::Node,
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
    util::{Result, Span},
};

/// Parses a model definition, consuming the complete input
///
/// This function **fails if the complete input is not consumed**.
pub fn parse_complete(
    input: Span<'_>,
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
/// This function parses a model that consists of:
/// 1. Optional leading whitespace and newlines
/// 2. Optional note at the beginning
/// 3. Zero or more top-level declarations
/// 4. Zero or more sections, each containing declarations
///
/// The function uses error recovery to continue parsing even when individual
/// declarations or sections fail, allowing multiple syntax errors to be
/// reported in a single pass.
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns a model node if parsing succeeds, or a partial result with errors
/// if some parts failed to parse.
///
/// # Error Recovery Strategy
///
/// When a declaration fails to parse:
/// 1. The error is recorded
/// 2. The parser skips to the next line
/// 3. Parsing continues with the next declaration
///
/// When a section fails to parse:
/// 1. The section header error is recorded
/// 2. Any declarations within the failed section are moved to the top-level
/// 3. Parsing continues with the next section
fn model(
    input: Span<'_>,
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
/// This function parses zero or more top-level declarations. If it fails to
/// parse a declaration, it attempts to recover and continue parsing. This
/// allows for multiple syntax errors to be found in the model.
///
/// # Recovery Strategy
///
/// When a declaration fails to parse:
/// 1. Check if the next token is a section header or end of file
/// 2. If so, stop parsing declarations and return accumulated results
/// 3. If not, record the error and skip to the next line
/// 4. Continue parsing with the next declaration
///
/// The function handles consecutive errors by avoiding duplicate error
/// reporting for lines that might be continuations of previous failed
/// declarations (e.g., multi-line piecewise functions).
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns a tuple containing:
/// - The remaining unparsed input
/// - Vector of successfully parsed declaration nodes
/// - Vector of parsing errors encountered
///
/// In addition, because it returns partial results, the results may be used
/// in order to determine other partial information, such as the associated
/// units of the declarations that were successfully parsed.
fn parse_decls(input: Span<'_>) -> (Span<'_>, Vec<DeclNode>, Vec<ParserError>) {
    fn parse_decls_recur(
        input: Span<'_>,
        mut acc_decls: Vec<DeclNode>,
        mut acc_errors: Vec<ParserError>,
        last_was_error: bool,
    ) -> (Span<'_>, Vec<DeclNode>, Vec<ParserError>) {
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
///
/// This function parses zero or more sections in the model. Each section
/// consists of a section header followed by declarations. If a section
/// header fails to parse, any declarations that follow are treated as
/// top-level declarations.
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns a tuple containing:
/// - The remaining unparsed input
/// - Vector of successfully parsed section nodes
/// - Vector of declarations that were parsed but couldn't be assigned to a section
/// - Vector of parsing errors encountered
fn parse_sections(
    input: Span<'_>,
) -> (Span<'_>, Vec<SectionNode>, Vec<DeclNode>, Vec<ParserError>) {
    fn parse_sections_recur(
        input: Span<'_>,
        mut acc_sections: Vec<SectionNode>,
        mut acc_decls: Vec<DeclNode>,
        mut acc_errors: Vec<ParserError>,
    ) -> (Span<'_>, Vec<SectionNode>, Vec<DeclNode>, Vec<ParserError>) {
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
///
/// A section consists of:
/// 1. A section header (`section <label>`)
/// 2. Optional note
/// 3. Zero or more declarations
///
/// If there is no section header, this function returns `None`, indicating that
/// no section was found.
///
/// Otherwise, this function returns a tuple containing the section and the
/// errors that occurred while parsing the section, if any.
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns `None` if no section header is found, or `Some` containing:
/// - The remaining unparsed input
/// - Either a successfully parsed section node or a vector of declarations
/// - Vector of parsing errors encountered
fn parse_section(input: Span<'_>) -> Option<(Span<'_>, SectionResult, SectionErrors)> {
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
///
/// A section header has the format: `section label` followed by a newline.
/// The function parses the `section` keyword, extracts the label identifier,
/// and ensures the header is properly terminated with a newline.
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns a section header node containing the parsed label, or an error
/// if the header is malformed.
///
/// # Error Conditions
///
/// - **Missing label**: When the `section` keyword is followed by whitespace or newline
/// - **Missing newline**: When the label is not followed by a proper line terminator
/// - **Invalid label**: When the label contains invalid characters
fn parse_section_header(input: Span<'_>) -> Result<'_, SectionHeaderNode, ParserError> {
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
///
/// # Arguments
///
/// * `input` - The input span to skip from
///
/// # Returns
///
/// Returns the remaining input after skipping to the next line.
fn skip_to_next_line_with_content(input: Span<'_>) -> Span<'_> {
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
    use oneil_ast::declaration::Decl;

    #[test]
    fn test_empty_model() {
        let input = Span::new_extra("", Config::default());
        let (rest, model) = parse_complete(input).expect("should parse empty model");
        assert!(model.note().is_none());
        assert!(model.decls().is_empty());
        assert!(model.sections().is_empty());
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_model_with_note() {
        let input = Span::new_extra("~ This is a note\n", Config::default());
        let (rest, model) = parse_complete(input).expect("should parse model with note");
        assert!(model.note().is_some());
        assert!(model.decls().is_empty());
        assert!(model.sections().is_empty());
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_model_with_import() {
        let input = Span::new_extra("import foo\n", Config::default());
        let (rest, model) = parse_complete(input).expect("should parse model with import");
        assert!(model.note().is_none());
        assert_eq!(model.decls().len(), 1);
        match &model.decls()[0].node_value() {
            Decl::Import(import_node) => assert_eq!(import_node.path().node_value(), &"foo"),
            _ => panic!("Expected import declaration"),
        }
        assert!(model.sections().is_empty());
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_use_without_as() {
        let input = Span::new_extra("use foo\n", Config::default());
        let (rest, model) = parse_complete(input).expect("should parse model with use declaration");
        assert!(model.note().is_none());
        assert_eq!(model.decls().len(), 1);
        match &model.decls()[0].node_value() {
            Decl::UseModel(use_model_node) => {
                let use_model_info = use_model_node.model_info();
                assert_eq!(use_model_info.top_component().as_str(), "foo");
                assert_eq!(use_model_info.subcomponents().len(), 0);
                assert_eq!(use_model_info.get_alias().as_str(), "foo");
            }
            _ => panic!("Expected use declaration"),
        }
        assert!(model.sections().is_empty());
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_model_with_section() {
        let input = Span::new_extra("section foo\nimport bar\n", Config::default());
        let (rest, model) = parse_complete(input).expect("should parse model with section");
        assert!(model.note().is_none());
        assert!(model.decls().is_empty());
        assert_eq!(model.sections().len(), 1);
        let section = &model.sections()[0];
        assert_eq!(section.header().label().as_str(), "foo");
        assert_eq!(section.decls().len(), 1);
        match &section.decls()[0].node_value() {
            Decl::Import(import_node) => assert_eq!(import_node.path().node_value(), "bar"),
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
        let (rest, model) =
            parse_complete(input).expect("should parse model with multiple sections");
        assert!(model.note().is_none());
        assert!(model.decls().is_empty());
        assert_eq!(model.sections().len(), 2);

        let section1 = &model.sections()[0];
        assert_eq!(section1.header().label().as_str(), "foo");
        assert_eq!(section1.decls().len(), 1);
        match &section1.decls()[0].node_value() {
            Decl::Import(import_node) => assert_eq!(import_node.path().node_value(), "bar"),
            _ => panic!("Expected import declaration"),
        }

        let section2 = &model.sections()[1];
        assert_eq!(section2.header().label().as_str(), "baz");
        assert_eq!(section2.decls().len(), 1);
        match &section2.decls()[0].node_value() {
            Decl::Import(import_node) => assert_eq!(import_node.path().node_value(), "qux"),
            _ => panic!("Expected import declaration"),
        }

        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_parse_complete_empty_model_success() {
        let input = Span::new_extra("\n", Config::default());
        let (rest, model) = parse_complete(input).expect("should parse empty model");
        assert!(model.note().is_none());
        assert!(model.decls().is_empty());
        assert!(model.sections().is_empty());
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_parse_complete_with_declarations_success() {
        let input = Span::new_extra("import foo\nimport bar\n", Config::default());
        let (rest, model) = parse_complete(input).expect("should parse model with declarations");
        assert_eq!(model.decls().len(), 2);
        match &model.decls()[0].node_value() {
            Decl::Import(import_node) => assert_eq!(import_node.path().node_value(), "foo"),
            _ => panic!("Expected import declaration"),
        }
        match &model.decls()[1].node_value() {
            Decl::Import(import_node) => assert_eq!(import_node.path().node_value(), "bar"),
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
        let (rest, model) = parse_complete(input).expect("should parse empty model");
        assert!(model.note().is_none());
        assert!(model.decls().is_empty());
        assert!(model.sections().is_empty());
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_parse_model_with_parameters() {
        let input = Span::new_extra(
            "1st parameter: x = 1\n2nd parameter: y = 2\n",
            Config::default(),
        );
        let (rest, model) = parse_complete(input).expect("should parse model with parameters");
        assert!(model.note().is_none());
        assert_eq!(model.decls().len(), 2);
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_parse_model_with_section_and_declarations() {
        let input = Span::new_extra(
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

                assert_eq!(model.decls().len(), 1);
                assert_eq!(model.sections().len(), 1);
                assert_eq!(model.sections()[0].header().label().as_str(), "My Section");
                assert_eq!(model.sections()[0].decls().len(), 1);

                assert_eq!(errors.len(), 4);
            }
            _ => panic!("Expected an error with incomplete input"),
        }
    }

    mod error_tests {
        use super::*;
        use crate::error::reason::{ExpectKind, IncompleteKind, ParserErrorReason, SectionKind};

        mod general_error_tests {
            use super::*;

            #[test]
            fn test_parse_complete_with_remaining_input() {
                let input = Span::new_extra("import foo\nrest", Config::default());
                let result = parse_complete(input);
                match result {
                    Err(nom::Err::Failure(e)) => {
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
                    _ => panic!("Expected error for remaining input"),
                }
            }
        }

        mod section_error_tests {
            use super::*;

            #[test]
            fn test_section_missing_label() {
                let input = Span::new_extra("section\nimport foo\n", Config::default());
                let result = parse_complete(input);
                match result {
                    Err(nom::Err::Failure(e)) => {
                        let model = e.partial_result;
                        let errors = e.errors;

                        assert_eq!(model.decls().len(), 1);
                        assert_eq!(errors.len(), 1);
                        assert_eq!(errors[0].error_offset, 7);
                        match errors[0].reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Section(SectionKind::MissingLabel),
                                cause,
                            } => {
                                assert_eq!(cause, AstSpan::new(0, 7, 0));
                            }
                            _ => panic!("Unexpected reason {:?}", errors[0].reason),
                        }
                    }
                    _ => panic!("Expected error for missing section label"),
                }
            }

            #[test]
            fn test_section_missing_end_of_line() {
                let input = Span::new_extra("section foo :\n import foo", Config::default());
                let result = parse_complete(input);
                match result {
                    Err(nom::Err::Failure(e)) => {
                        let model = e.partial_result;
                        let errors = e.errors;

                        assert_eq!(model.decls().len(), 1);
                        assert_eq!(errors.len(), 1);
                        assert_eq!(errors[0].error_offset, 12);
                        match errors[0].reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Section(SectionKind::MissingEndOfLine),
                                cause,
                            } => {
                                assert_eq!(cause, AstSpan::new(8, 3, 1));
                            }
                            _ => panic!("Unexpected reason {:?}", errors[0].reason),
                        }
                    }
                    _ => panic!("Expected error for missing section label, got {result:?}"),
                }
            }

            #[test]
            fn test_section_with_invalid_declaration() {
                let input = Span::new_extra("section foo\nimport\n", Config::default());
                let result = parse_complete(input);
                match result {
                    Err(nom::Err::Failure(e)) => {
                        let model = e.partial_result;
                        let errors = e.errors;

                        assert_eq!(model.sections().len(), 1);
                        assert_eq!(model.sections()[0].decls().len(), 0);
                        assert_eq!(errors.len(), 1);
                        assert_eq!(errors[0].error_offset, 18);
                        match errors[0].reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Decl(_),
                                cause,
                            } => {
                                assert_eq!(cause, AstSpan::new(12, 6, 0));
                            }
                            _ => panic!("Unexpected reason {:?}", errors[0].reason),
                        }
                    }
                    _ => panic!("Expected error for invalid declaration in section"),
                }
            }
        }

        mod declaration_error_tests {
            use super::*;

            #[test]
            fn test_import_missing_path() {
                let input = Span::new_extra("import\n", Config::default());
                let result = parse_complete(input);
                match result {
                    Err(nom::Err::Failure(e)) => {
                        let model = e.partial_result;
                        let errors = e.errors;

                        assert_eq!(model.decls().len(), 0);
                        assert_eq!(errors.len(), 1);
                        assert_eq!(errors[0].error_offset, 6);
                        match errors[0].reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Decl(_),
                                cause,
                            } => {
                                assert_eq!(cause, AstSpan::new(0, 6, 0));
                            }
                            _ => panic!("Unexpected reason {:?}", errors[0].reason),
                        }
                    }
                    _ => panic!("Expected error for missing import path"),
                }
            }

            #[test]
            fn test_parameter_missing_equals() {
                let input = Span::new_extra("X: x\n", Config::default());
                let result = parse_complete(input);
                match result {
                    Err(nom::Err::Failure(e)) => {
                        let model = e.partial_result;
                        let errors = e.errors;

                        assert_eq!(model.decls().len(), 0);
                        assert_eq!(errors.len(), 1);
                        assert_eq!(errors[0].error_offset, 4);
                        match errors[0].reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Parameter(_),
                                cause,
                            } => {
                                assert_eq!(cause, AstSpan::new(3, 1, 0));
                            }
                            _ => panic!("Unexpected reason {:?}", errors[0].reason),
                        }
                    }
                    _ => panic!("Expected error for missing equals in parameter"),
                }
            }

            #[test]
            fn test_parameter_missing_value() {
                let input = Span::new_extra("X: x =\n", Config::default());
                let result = parse_complete(input);
                match result {
                    Err(nom::Err::Failure(e)) => {
                        let model = e.partial_result;
                        let errors = e.errors;

                        assert_eq!(model.decls().len(), 0);
                        assert_eq!(errors.len(), 1);
                        assert_eq!(errors[0].error_offset, 6);
                        match errors[0].reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Parameter(_),
                                cause,
                            } => {
                                assert_eq!(cause, AstSpan::new(5, 1, 0));
                            }
                            _ => panic!("Unexpected reason {:?}", errors[0].reason),
                        }
                    }
                    _ => panic!("Expected error for missing value in parameter"),
                }
            }

            #[test]
            fn test_test_missing_colon() {
                let input = Span::new_extra("test x > 0\n", Config::default());
                let result = parse_complete(input);
                match result {
                    Err(nom::Err::Failure(e)) => {
                        let model = e.partial_result;
                        let errors = e.errors;

                        assert_eq!(model.decls().len(), 0);
                        assert_eq!(errors.len(), 1);
                        assert_eq!(errors[0].error_offset, 5);
                        match errors[0].reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Test(_),
                                cause,
                            } => {
                                assert_eq!(cause, AstSpan::new(0, 4, 1));
                            }
                            _ => panic!("Unexpected reason {:?}", errors[0].reason),
                        }
                    }
                    _ => panic!("Expected error for missing colon in test"),
                }
            }
        }

        mod note_error_tests {
            use super::*;

            #[test]
            fn test_unterminated_note() {
                let input = Span::new_extra("~~~\nThis is an unterminated note", Config::default());
                let result = parse_complete(input);
                match result {
                    Err(nom::Err::Failure(e)) => {
                        let model = e.partial_result;
                        let errors = e.errors;

                        assert_eq!(model.decls().len(), 0);
                        assert_eq!(errors.len(), 1);
                        match errors[0].reason {
                            ParserErrorReason::TokenError(_) => {}
                            _ => panic!("Unexpected reason {:?}", errors[0].reason),
                        }
                    }
                    _ => panic!("Expected error for unterminated note"),
                }
            }
        }

        mod recovery_error_tests {
            use super::*;

            #[test]
            fn test_multiple_declaration_errors() {
                let input = Span::new_extra("import\nuse\nX: x\n", Config::default());
                let result = parse_complete(input);
                match result {
                    Err(nom::Err::Failure(e)) => {
                        let model = e.partial_result;
                        let errors = e.errors;

                        assert_eq!(model.decls().len(), 0);
                        assert_eq!(errors.len(), 3);

                        // All errors should be declaration-related
                        for error in &errors {
                            match error.reason {
                                ParserErrorReason::Incomplete {
                                    kind: IncompleteKind::Decl(_) | IncompleteKind::Parameter(_),
                                    ..
                                } => {}
                                _ => panic!("Expected declaration error, got {:?}", error.reason),
                            }
                        }
                    }
                    _ => panic!("Expected error for multiple declaration errors"),
                }
            }

            #[test]
            fn test_section_with_multiple_errors() {
                let input = Span::new_extra("section foo\nimport\nuse\nX: x\n", Config::default());
                let result = parse_complete(input);
                match result {
                    Err(nom::Err::Failure(e)) => {
                        let model = e.partial_result;
                        let errors = e.errors;

                        assert_eq!(model.sections().len(), 1);
                        assert_eq!(model.sections()[0].decls().len(), 0);
                        assert_eq!(errors.len(), 3);

                        // Should have three declaration errors
                        let mut decl_errors = 0;
                        let mut section_errors = 0;

                        for error in &errors {
                            match error.reason {
                                ParserErrorReason::Incomplete {
                                    kind: IncompleteKind::Decl(_) | IncompleteKind::Parameter(_),
                                    ..
                                } => decl_errors += 1,
                                ParserErrorReason::Incomplete {
                                    kind: IncompleteKind::Section(_),
                                    ..
                                } => section_errors += 1,
                                _ => panic!("Unexpected error type {:?}", error.reason),
                            }
                        }

                        assert_eq!(decl_errors, 3);
                        assert_eq!(section_errors, 0);
                    }
                    _ => panic!("Expected error for section with multiple errors"),
                }
            }

            #[test]
            fn test_mixed_valid_and_invalid_declarations() {
                let input = Span::new_extra(
                    "import valid\nimport\nuse foo as bar\nuse invalid.\n",
                    Config::default(),
                );
                let result = parse_complete(input);
                match result {
                    Err(nom::Err::Failure(e)) => {
                        let model = e.partial_result;
                        let errors = e.errors;

                        // Should have successfully parsed some declarations
                        assert_eq!(model.decls().len(), 2);
                        assert_eq!(errors.len(), 2);

                        // Check that the valid declarations were parsed
                        match &model.decls()[0].node_value() {
                            Decl::Import(import_node) => {
                                assert_eq!(import_node.path().node_value(), "valid");
                            }
                            _ => panic!("Expected import declaration"),
                        }
                        match &model.decls()[1].node_value() {
                            Decl::UseModel(use_node) => {
                                let alias = use_node.model_info().get_alias();
                                assert_eq!(alias.as_str(), "bar");
                            }
                            _ => panic!("Expected use model declaration"),
                        }
                    }
                    _ => panic!("Expected error for mixed valid and invalid declarations"),
                }
            }
        }
    }
}
