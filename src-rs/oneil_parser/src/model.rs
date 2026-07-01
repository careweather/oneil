//! Parser for model definitions in an Oneil program.
//!
//! The parser uses error recovery to continue parsing even when individual
//! declarations or sections fail, allowing multiple syntax errors to be
//! reported in a single pass.

use std::result::Result as StdResult;

use std::sync::Arc;

use nom::{
    Input, Parser as _,
    bytes::complete::take_while,
    combinator::{eof, opt, value},
};
use oneil_ast::{
    DeclNode, Model, ModelNode, Node, Section, SectionHeader, SectionHeaderNode, SectionLabelNode,
    SectionNode,
};
use oneil_shared::span::Span;

use crate::{
    declaration::{parse as parse_decl, parse_design_target_line},
    error::{
        ParserError,
        parser_trait::ErrorHandlingParser,
        partial::ParserPartialModelError,
        reason::{ExpectKind, ParserErrorReason},
    },
    note::parse as parse_note,
    token::{
        keyword::section,
        naming::label,
        structure::{end_of_line, start_of_file},
    },
    util::{InputSpan, Result, source_location_from},
};

/// Parses a model definition, consuming the complete input
///
/// This function **fails if the complete input is not consumed**.
pub fn parse_complete(input: InputSpan<'_>) -> Result<'_, ModelNode, Box<ParserPartialModelError>> {
    let (rest, model) = model(input)?;
    let result = eof(rest);

    match result {
        Ok((rest, _)) => Ok((rest, model)),
        Err(nom::Err::Error(e)) => Err(nom::Err::Failure(Box::new(ParserPartialModelError::new(
            model,
            vec![e],
        )))),
        _ => unreachable!(),
    }
}

/// Parses a complete model definition with error recovery.
///
/// The function uses error recovery to continue parsing even when individual
/// declarations or sections fail, allowing multiple syntax errors to be
/// reported in a single pass.
fn model(input: InputSpan<'_>) -> Result<'_, ModelNode, Box<ParserPartialModelError>> {
    // Capture path and source from input before it is moved into the parser.
    let path = Arc::clone(&input.extra.path);
    let source = Arc::clone(&input.extra.source);
    // Capture the start location before consuming input.
    let model_span_start = source_location_from(&input);

    let (rest, start_of_file_token) = start_of_file(input).expect("should always parse");

    let (rest, note) = opt(parse_note)
        .parse(rest.clone())
        .map_err(|error| handle_model_note_failure(rest, error))?;

    let (rest, mut decls, decl_errors) = parse_top_level_decls(rest);
    let (rest, sections, decls_without_section, section_errors) = parse_sections(rest);

    // for any decls where the section header parsing failed, add them to the top-level decls
    decls.extend(decls_without_section);

    let errors = [decl_errors, section_errors].concat();

    // get the last span/whitespace span of the model
    let start_of_file_spans = (
        start_of_file_token.lexeme_span,
        start_of_file_token.whitespace_span,
    );
    let last_note_spans = note
        .as_ref()
        .map(|note| (note.span().clone(), note.whitespace_span().clone()));
    let last_decl_spans = decls
        .last()
        .map(|decl| (decl.span().clone(), decl.whitespace_span().clone()));
    let last_section_spans = sections
        .last()
        .map(|section| (section.span().clone(), section.whitespace_span().clone()));

    let (last_node_span, last_node_whitespace_span) =
        [last_note_spans, last_decl_spans, last_section_spans]
            .into_iter()
            .flatten()
            .max_by(|a, b| {
                let a_end = a.0.end();
                let b_end = b.0.end();
                a_end
                    .partial_cmp(b_end)
                    .expect("should always compare because they are from the same file")
            })
            .unwrap_or(start_of_file_spans);

    // calculate the end of the model span
    let model_span_end = *last_node_span.end();

    // build the model span
    let model_span = Span::new(
        model_span_start,
        model_span_end,
        Arc::clone(&path),
        Arc::clone(&source),
    );

    // calculate the whitespace span of the model
    // (if there was no last node, use the model span)
    let model_whitespace_span = last_node_whitespace_span;

    let model_node = Node::new(
        Model::new(note, decls, sections),
        model_span,
        model_whitespace_span,
    );

    // if there were errors, return a partial result
    if errors.is_empty() {
        Ok((rest, model_node))
    } else {
        Err(nom::Err::Failure(Box::new(ParserPartialModelError::new(
            model_node, errors,
        ))))
    }
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "the parameter *is* in fact consumed..."
)]
fn handle_model_note_failure(
    input: InputSpan<'_>,
    error: nom::Err<ParserError>,
) -> nom::Err<Box<ParserPartialModelError>> {
    match error {
        nom::Err::Error(e) => panic!("should not be able to fail with an error: {e:?}"),

        nom::Err::Failure(e) => {
            let source_location = source_location_from(&input);
            let path = Arc::clone(&input.extra.path);
            let source_text = Arc::clone(&input.extra.source);
            let span = Span::empty(source_location, path, source_text);
            let model_node = ModelNode::new(Model::empty(), span.clone(), span);
            let errors = vec![e];
            let partial_error = ParserPartialModelError::new(model_node, errors);
            nom::Err::Failure(Box::new(partial_error))
        }

        nom::Err::Incomplete(e) => nom::Err::Incomplete(e),
    }
}

/// Parses top-level declarations after the optional note.
///
/// For `.one` design files (when [`Config::require_design_header`](crate::Config::require_design_header)
/// is set), a `design <model>` line is parsed eagerly before other declarations; missing or
/// misplaced headers are reported and the rest of the file is still parsed for additional
/// diagnostics.
fn parse_top_level_decls(input: InputSpan<'_>) -> (InputSpan<'_>, Vec<DeclNode>, Vec<ParserError>) {
    if input.extra.require_design_header {
        parse_design_file_decls(&input)
    } else {
        parse_decls_recur(input, vec![], vec![], false)
    }
}

/// Parses the body of a `.one` design bundle.
///
/// On a successful `design <model>` header, sets
/// [`Config::allow_design_shorthand`](crate::Config::allow_design_shorthand) on the input span
/// so subsequent declarations may use shorthand parameter assignments. On failure, distinguishes
/// between:
///
/// - whitespace-only / empty input → [`ParserError::design_header_missing`] ("missing design target")
/// - any other content where the first declaration is not `design` → [`ParserError::design_header_not_first`]
fn parse_design_file_decls<'a>(
    input: &InputSpan<'a>,
) -> (InputSpan<'a>, Vec<DeclNode>, Vec<ParserError>) {
    match parse_design_target_line(input.clone()) {
        Ok((mut rest, node)) => {
            rest.extra.allow_design_shorthand = true;
            parse_decls_recur(rest, vec![node], vec![], false)
        }
        Err(nom::Err::Error(e) | nom::Err::Failure(e)) => {
            let rest = skip_to_next_line_with_content(input.clone(), e.error_offset);
            let (trimmed, _) = take_while::<_, _, nom::error::Error<_>>(char::is_whitespace)
                .parse(input.clone())
                .expect("take_while whitespace always succeeds");
            let span = Span::empty(
                source_location_from(&trimmed),
                Arc::clone(&trimmed.extra.path),
                Arc::clone(&trimmed.extra.source),
            );
            let header_error = if input.fragment().chars().all(char::is_whitespace) {
                ParserError::design_header_missing(span)
            } else {
                ParserError::design_header_not_first(span)
            };
            parse_decls_recur(rest, vec![], vec![header_error], true)
        }
        Err(nom::Err::Incomplete(_)) => unreachable!("design header uses complete parsers"),
    }
}

/// Attempts to parse declarations with error recovery
///
/// The function handles consecutive errors by avoiding duplicate error
/// reporting for lines that might be continuations of previous failed
/// declarations (e.g., multi-line piecewise functions).
///
/// Whether design-body shorthand (`p.r = expr`) is enabled is read from
/// [`Config::allow_design_shorthand`](crate::Config::allow_design_shorthand) on the input span's
/// `extra`, set by [`parse_design_file_decls`] after a successful `design <model>` line.
fn parse_decls_recur(
    input: InputSpan<'_>,
    mut acc_decls: Vec<DeclNode>,
    mut acc_errors: Vec<ParserError>,
    last_was_error: bool,
) -> (InputSpan<'_>, Vec<DeclNode>, Vec<ParserError>) {
    let result = parse_decl(input.clone());

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
            if section.or(end_of_file).parse(input.clone()).is_ok() {
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
                acc_errors.push(e.clone());
            }

            // All declarations must be terminated by an end of line, so we
            // assume that the declaration parsing error is for a declaration
            // that ends at the end of the line
            let next_line = skip_to_next_line_with_content(input, e.error_offset);

            parse_decls_recur(next_line, acc_decls, acc_errors, true)
        }
        Err(nom::Err::Incomplete(_needed)) => (input, acc_decls, acc_errors),
    }
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
        let section_result = parse_section(input.clone());

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
    let section_header_result = parse_section_header(input.clone());

    let (rest, header, mut errors) = match section_header_result {
        Ok((rest, header)) => (rest, Some(header), vec![]),
        Err(nom::Err::Error(_e)) => {
            // No section header was found, so we return None
            return None;
        }
        Err(nom::Err::Failure(e)) => {
            // There was a problem with the section header, so we keep the error and skip to the next line
            let rest = skip_to_next_line_with_content(input, e.error_offset);
            (rest, None, vec![e])
        }
        Err(nom::Err::Incomplete(_needed)) => (input, None, vec![]),
    };

    let (rest, note) = opt(parse_note)
        .parse(rest)
        .expect("should always parse because its optional");

    let (rest, decls, decl_errors) = parse_decls_recur(rest, vec![], vec![], false);
    errors.extend(decl_errors);

    match header {
        Some(header) => {
            let section_start_span = header.span().clone();
            let (section_end_span, section_whitespace_span): (Span, Span) =
                match (&note, decls.last()) {
                    (_, Some(decl)) => (decl.span().clone(), decl.whitespace_span().clone()),
                    (Some(note), _) => (note.span().clone(), note.whitespace_span().clone()),
                    (_, _) => (header.span().clone(), header.whitespace_span().clone()),
                };

            let section_span = Span::from_start_and_end(&section_start_span, &section_end_span);

            let section_node = Node::new(
                Section::new(header, note, decls),
                section_span,
                section_whitespace_span,
            );

            Some((rest, Ok(section_node), errors))
        }
        // if there was a problem with the section header, return the decls parsed so that
        // they can be merged with the top-level decls
        None => Some((rest, Err(decls), errors)),
    }
}

/// Parses a section header with its label
fn parse_section_header(input: InputSpan<'_>) -> Result<'_, SectionHeaderNode, ParserError> {
    let (rest, section_token) = section.convert_errors().parse(input)?;

    let (rest, label_token) = label
        .or_fail_with(ParserError::section_missing_label(
            section_token.lexeme_span.clone(),
        ))
        .parse(rest)?;
    let label_lexeme_span = label_token.lexeme_span.clone();
    let label_node = SectionLabelNode::from(label_token);

    let (rest, end_of_line_token) = end_of_line
        .or_fail_with(ParserError::section_missing_end_of_line(label_lexeme_span))
        .parse(rest)?;

    let label_span =
        Span::from_start_and_end(&section_token.lexeme_span, &end_of_line_token.lexeme_span);
    let label_whitespace_span = end_of_line_token.whitespace_span;
    let header_node = Node::new(
        SectionHeader::new(label_node),
        label_span,
        label_whitespace_span,
    );

    Ok((rest, header_node))
}

/// Attempts to recover from a parsing error by skipping to the next line
///
/// This function is used for error recovery when parsing declarations or
/// section headers. It skips all characters until it finds a newline or
/// end of file, then consumes the newline character itself.
///
/// It also optionally skips a note that follows the line break.
fn skip_to_next_line_with_content(input: InputSpan<'_>, error_offset: usize) -> InputSpan<'_> {
    // advance to the error offset
    let chars_to_skip = error_offset - input.location_offset();
    let rest = input.take_from(chars_to_skip);

    // first, just try parsing a note
    if let Ok((rest, _)) = parse_note.parse(rest) {
        return rest;
    }

    // otherwise, skip to the next line, then try parsing a note
    let (rest, _) = take_while::<_, _, nom::error::Error<_>>(|c| c != '\n')
        .parse(input)
        .expect("should never fail");

    let (rest, _) = end_of_line
        .parse(rest)
        .expect("should always parse either a line break or EOF");

    opt(parse_note)
        .parse(rest.clone())
        // if the note parsing failed, return the previous `rest`
        .map_or_else(|_| rest, |(rest, _)| rest)
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

        let Decl::Import(import_node) = model.decls()[0].clone().take_value() else {
            panic!("Expected import declaration");
        };
        assert_eq!(import_node.path().as_str(), "foo");

        assert!(model.sections().is_empty());
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn submodel_without_as() {
        let input = InputSpan::new_extra("submodel foo\n", Config::default());
        let (rest, model) =
            parse_complete(input).expect("should parse model with submodel declaration");
        assert!(model.note().is_none());
        assert_eq!(model.decls().len(), 1);

        let Decl::Submodel(submodel_node) = model.decls()[0].clone().take_value() else {
            panic!("Expected submodel declaration");
        };

        let submodel_info = submodel_node.model_info();
        assert_eq!(submodel_info.top_component().as_str(), "foo");
        assert_eq!(submodel_info.subcomponents().len(), 0);
        assert_eq!(submodel_info.get_alias().as_str(), "foo");

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
        let Decl::Import(import_node) = section.decls()[0].clone().take_value() else {
            panic!("Expected import declaration");
        };
        assert_eq!(import_node.path().as_str(), "bar");
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
        let Decl::Import(import_node) = section1.decls()[0].clone().take_value() else {
            panic!("Expected import declaration");
        };
        assert_eq!(import_node.path().as_str(), "bar");

        let section2 = &model.sections()[1];
        assert_eq!(section2.header().label().as_str(), "baz");
        assert_eq!(section2.decls().len(), 1);
        let Decl::Import(import_node) = section2.decls()[0].clone().take_value() else {
            panic!("Expected import declaration");
        };
        assert_eq!(import_node.path().as_str(), "qux");

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
        let Decl::Import(import_node) = model.decls()[0].clone().take_value() else {
            panic!("Expected import declaration");
        };
        assert_eq!(import_node.path().as_str(), "foo");
        let Decl::Import(import_node) = model.decls()[1].clone().take_value() else {
            panic!("Expected import declaration");
        };
        assert_eq!(import_node.path().as_str(), "bar");
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
            submodel foo as bar

            from foo submodel as baz # missing `submodel` part

            X: x = 1 + # incomplete

            section My Section

            submodel foo as bar

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

        let model = &e.partial_result;
        let errors = &e.error_collection;

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

            let model = &e.partial_result;
            let errors = &e.error_collection;

            assert_eq!(model.decls().len(), 1);
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].error_offset, 15);
            assert_eq!(
                errors[0].reason,
                ParserErrorReason::Expect(ExpectKind::Decl)
            );
        }
    }

    mod section_error {
        use crate::error::reason::{ExpectKind, IncompleteKind, ParserErrorReason, SectionKind};

        use super::*;

        #[test]
        fn section_missing_label() {
            let input = InputSpan::new_extra("section\nimport foo\n", Config::default());
            let result = parse_complete(input);
            let Err(nom::Err::Failure(e)) = result else {
                panic!("Expected error for missing section label");
            };

            let model = &e.partial_result;
            let errors = &e.error_collection;

            assert_eq!(model.decls().len(), 1);
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].error_offset, 7);
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Section(SectionKind::MissingLabel),
                ref cause,
            } = errors[0].reason
            else {
                panic!("Unexpected reason {:?}", errors[0].reason);
            };

            assert_eq!(cause.start().offset, 0);
            assert_eq!(cause.end().offset, 7);
        }

        #[test]
        fn section_missing_end_of_line() {
            let input = InputSpan::new_extra("section foo :\n import foo", Config::default());
            let result = parse_complete(input);
            let Err(nom::Err::Failure(e)) = result else {
                panic!("Expected error for missing section label, got {result:?}");
            };

            let model = &e.partial_result;
            let errors = &e.error_collection;

            assert_eq!(model.decls().len(), 1);
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].error_offset, 12);
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Section(SectionKind::MissingEndOfLine),
                ref cause,
            } = errors[0].reason
            else {
                panic!("Unexpected reason {:?}", errors[0].reason);
            };

            assert_eq!(cause.start().offset, 8);
            assert_eq!(cause.end().offset, 11);
        }

        #[test]
        fn section_with_invalid_declaration() {
            let input = InputSpan::new_extra("section foo\nimport\n", Config::default());
            let result = parse_complete(input);
            let Err(nom::Err::Failure(e)) = result else {
                panic!("Expected error for invalid declaration in section");
            };

            let model = &e.partial_result;
            let errors = &e.error_collection;

            assert_eq!(model.sections().len(), 1);
            assert_eq!(model.sections()[0].decls().len(), 0);
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].error_offset, 18);
            match &errors[0].reason {
                ParserErrorReason::Incomplete {
                    kind: IncompleteKind::Decl(_),
                    cause,
                } => {
                    assert_eq!(cause.start().offset, 12);
                    assert_eq!(cause.end().offset, 18);
                }
                ParserErrorReason::Expect(ExpectKind::Decl) => {}
                other @ (ParserErrorReason::Expect(_)
                | ParserErrorReason::Incomplete { .. }
                | ParserErrorReason::UnexpectedToken
                | ParserErrorReason::TokenError(_)
                | ParserErrorReason::NomError(_)) => panic!("Unexpected reason {other:?}"),
            }
        }
    }

    mod declaration_error {
        use crate::error::reason::{DeclKind, ExpectKind, IncompleteKind, ParserErrorReason};

        use super::*;

        #[test]
        fn design_file_requires_design_first_declaration() {
            let input = InputSpan::new_extra(
                "import foo\ndesign bar\n",
                Config {
                    require_design_header: true,
                    ..Config::default()
                },
            );
            let result = parse_complete(input);
            let Err(nom::Err::Failure(e)) = result else {
                panic!("Expected parse failure, got {result:?}");
            };
            assert!(e.error_collection.iter().any(|err| matches!(
                &err.reason,
                ParserErrorReason::Incomplete {
                    kind: IncompleteKind::Decl(DeclKind::DesignHeaderNotFirst),
                    ..
                }
            )));
        }

        #[test]
        fn design_file_rejects_empty_body() {
            let input = InputSpan::new_extra(
                "",
                Config {
                    require_design_header: true,
                    ..Config::default()
                },
            );
            let result = parse_complete(input);
            let Err(nom::Err::Failure(e)) = result else {
                panic!("Expected parse failure for empty design file, got {result:?}");
            };
            assert!(e.error_collection.iter().any(|err| matches!(
                &err.reason,
                ParserErrorReason::Incomplete {
                    kind: IncompleteKind::Decl(DeclKind::DesignHeaderMissing),
                    ..
                }
            )));
        }

        #[test]
        fn design_file_rejects_whitespace_only_body() {
            let input = InputSpan::new_extra(
                "   \n  \n",
                Config {
                    require_design_header: true,
                    ..Config::default()
                },
            );
            let result = parse_complete(input);
            let Err(nom::Err::Failure(e)) = result else {
                panic!("Expected parse failure for whitespace-only design file, got {result:?}");
            };
            assert!(e.error_collection.iter().any(|err| matches!(
                &err.reason,
                ParserErrorReason::Incomplete {
                    kind: IncompleteKind::Decl(DeclKind::DesignHeaderMissing),
                    ..
                }
            )));
        }

        #[test]
        fn design_file_rejects_duplicate_design() {
            let input = InputSpan::new_extra(
                "design foo\ndesign bar\n",
                Config {
                    require_design_header: true,
                    ..Config::default()
                },
            );
            let result = parse_complete(input);
            let Err(nom::Err::Failure(e)) = result else {
                panic!("Expected parse failure, got {result:?}");
            };
            assert!(e.error_collection.iter().any(|err| matches!(
                &err.reason,
                ParserErrorReason::Incomplete {
                    kind: IncompleteKind::Decl(DeclKind::DesignHeaderDuplicate),
                    ..
                }
            )));
        }

        #[test]
        fn section_body_accepts_apply() {
            let input =
                InputSpan::new_extra("section s\n  apply overlay to r\n", Config::default());
            let (_, model) = parse_complete(input).expect("parse");
            assert_eq!(model.sections().len(), 1);
            let section = &model.sections()[0];
            assert_eq!(section.decls().len(), 1);
            assert!(matches!(
                section.decls()[0].clone().take_value(),
                Decl::ApplyDesign(_)
            ));
        }

        #[test]
        fn design_header_rejected_without_require_design_header() {
            let input = InputSpan::new_extra("design foo\n", Config::default());
            let result = parse_complete(input);
            let Err(nom::Err::Failure(e)) = result else {
                panic!("Expected parse failure");
            };

            let errors = &e.error_collection;
            assert_eq!(errors.len(), 1);
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Decl(DeclKind::DesignHeaderWrongFile),
                ..
            } = errors[0].reason
            else {
                panic!("Unexpected reason {:?}", errors[0].reason);
            };
        }

        #[test]
        fn design_header_allowed_when_configured() {
            let input = InputSpan::new_extra(
                "design foo\n",
                Config {
                    require_design_header: true,
                    ..Config::default()
                },
            );
            let (rest, model) = parse_complete(input).expect("should parse design header");
            assert_eq!(model.decls().len(), 1);
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn apply_allowed_without_design_header_flag() {
            let input = InputSpan::new_extra("apply my_overlay to r\n", Config::default());
            let (rest, model) = parse_complete(input).expect("should parse apply");
            assert_eq!(model.decls().len(), 1);
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn import_missing_path() {
            let input = InputSpan::new_extra("import\n", Config::default());
            let result = parse_complete(input);
            let Err(nom::Err::Failure(e)) = result else {
                panic!("Expected error for missing import path");
            };

            let model = &e.partial_result;
            let errors = &e.error_collection;

            assert_eq!(model.decls().len(), 0);
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].error_offset, 6);
            match &errors[0].reason {
                ParserErrorReason::Incomplete {
                    kind: IncompleteKind::Decl(_),
                    cause,
                } => {
                    assert_eq!(cause.start().offset, 0);
                    assert_eq!(cause.end().offset, 6);
                }
                ParserErrorReason::Expect(ExpectKind::Decl) => {}
                other @ (ParserErrorReason::Expect(_)
                | ParserErrorReason::Incomplete { .. }
                | ParserErrorReason::UnexpectedToken
                | ParserErrorReason::TokenError(_)
                | ParserErrorReason::NomError(_)) => panic!("Unexpected reason {other:?}"),
            }
        }

        #[test]
        fn parameter_missing_equals() {
            let input = InputSpan::new_extra("X: x\n", Config::default());
            let result = parse_complete(input);
            let Err(nom::Err::Failure(e)) = result else {
                panic!("Expected error for missing equals in parameter");
            };

            let model = &e.partial_result;
            let errors = &e.error_collection;

            assert_eq!(model.decls().len(), 0);
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].error_offset, 4);
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Parameter(_),
                ref cause,
            } = errors[0].reason
            else {
                panic!("Unexpected reason {:?}", errors[0].reason);
            };

            assert_eq!(cause.start().offset, 3);
            assert_eq!(cause.end().offset, 4);
        }

        #[test]
        fn parameter_missing_value() {
            let input = InputSpan::new_extra("X: x =\n", Config::default());
            let result = parse_complete(input);
            let Err(nom::Err::Failure(e)) = result else {
                panic!("Expected error for missing value in parameter");
            };

            let model = &e.partial_result;
            let errors = &e.error_collection;

            assert_eq!(model.decls().len(), 0);
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].error_offset, 6);
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Parameter(_),
                ref cause,
            } = errors[0].reason
            else {
                panic!("Unexpected reason {:?}", errors[0].reason);
            };

            assert_eq!(cause.start().offset, 5);
            assert_eq!(cause.end().offset, 6);
        }

        #[test]
        fn missing_colon() {
            let input = InputSpan::new_extra("test x > 0\n", Config::default());
            let result = parse_complete(input);
            let Err(nom::Err::Failure(e)) = result else {
                panic!("Expected error for missing colon in test");
            };

            let model = &e.partial_result;
            let errors = &e.error_collection;

            assert_eq!(model.decls().len(), 0);
            assert_eq!(errors.len(), 1);
            // error_offset points to the token where `:` was expected (`x` at
            // offset 5 in `"test x > 0\n"`), not somewhere later in the expr.
            assert_eq!(errors[0].error_offset, 5);
            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Test(_),
                cause,
            } = &errors[0].reason
            else {
                panic!("Unexpected reason {:?}", errors[0].reason);
            };
            assert_eq!(cause.start().offset, 0);
            assert_eq!(cause.end().offset, 4);
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

            let model = &e.partial_result;
            let errors = &e.error_collection;

            assert_eq!(model.decls().len(), 0);
            assert_eq!(errors.len(), 1);
            let ParserErrorReason::TokenError(_) = errors[0].reason else {
                panic!("Unexpected reason {:?}", errors[0].reason);
            };
        }
    }

    mod recovery_error {
        use crate::error::reason::{ExpectKind, IncompleteKind, ParserErrorReason};

        use super::*;

        #[test]
        fn multiple_declaration_errors() {
            let input = InputSpan::new_extra("import\nsubmodel\nX: x\n", Config::default());
            let result = parse_complete(input);
            let Err(nom::Err::Failure(e)) = result else {
                panic!("Expected error for multiple declaration errors");
            };

            let model = &e.partial_result;
            let errors = &e.error_collection;

            assert_eq!(model.decls().len(), 0);
            assert_eq!(errors.len(), 3);

            // All errors should be declaration-related
            for error in errors {
                match &error.reason {
                    ParserErrorReason::Incomplete {
                        kind: IncompleteKind::Decl(_) | IncompleteKind::Parameter(_),
                        ..
                    }
                    | ParserErrorReason::Expect(ExpectKind::Decl) => {}
                    other @ (ParserErrorReason::Expect(_)
                    | ParserErrorReason::Incomplete { .. }
                    | ParserErrorReason::UnexpectedToken
                    | ParserErrorReason::TokenError(_)
                    | ParserErrorReason::NomError(_)) => {
                        panic!("Expected declaration error, got {other:?}")
                    }
                }
            }
        }

        #[test]
        fn section_with_multiple_errors() {
            let input =
                InputSpan::new_extra("section foo\nimport\nsubmodel\nX: x\n", Config::default());
            let result = parse_complete(input);
            let Err(nom::Err::Failure(e)) = result else {
                panic!("Expected error for section with multiple errors");
            };

            let model = &e.partial_result;
            let errors = &e.error_collection;

            assert_eq!(model.sections().len(), 1);
            assert_eq!(model.sections()[0].decls().len(), 0);
            assert_eq!(errors.len(), 3);
        }

        #[test]
        fn mixed_valid_and_invalid_declarations() {
            let input = InputSpan::new_extra(
                "import valid\nimport\nsubmodel foo as bar\nsubmodel invalid.\n",
                Config::default(),
            );
            let result = parse_complete(input);
            let Err(nom::Err::Failure(e)) = result else {
                panic!("Expected error for mixed valid and invalid declarations");
            };

            let model = &e.partial_result;
            let errors = &e.error_collection;

            // Should have successfully parsed some declarations
            assert_eq!(model.decls().len(), 2);
            assert_eq!(errors.len(), 2);

            // Check that the valid declarations were parsed
            let Decl::Import(import_node) = model.decls()[0].clone().take_value() else {
                panic!("Expected import declaration");
            };
            assert_eq!(import_node.path().as_str(), "valid");
            let Decl::Submodel(submodel_node) = model.decls()[1].clone().take_value() else {
                panic!("Expected submodel declaration");
            };
            let alias = submodel_node.model_info().get_alias();
            assert_eq!(alias.as_str(), "bar");
        }
    }
}
