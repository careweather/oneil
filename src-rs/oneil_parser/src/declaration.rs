//! Parser for declarations in an Oneil program.

use nom::{
    Parser as _,
    branch::alt,
    combinator::{all_consuming, opt},
    multi::many0,
};

use oneil_ast::{
    ApplyDesign, ApplyDesignNode, Decl, DeclNode, DesignParameter, DesignTarget, Directory,
    DirectoryNode, IdentifierNode, Import, ModelInfo, ModelInfoNode, ModelKind, Node,
    ParameterLabelNode, RenderNameNode, SubmodelDecl, SubmodelList, SubmodelListNode,
};
use oneil_shared::{labels::RenderName, span::Span};

use crate::{
    error::{ParserError, parser_trait::ErrorHandlingParser, reason::SubmodelKeyword},
    note::parse as parse_note,
    parameter::{parse as parse_parameter, parse_parameter_value, performance_marker, trace_level},
    test::parse as parse_test,
    token::{
        keyword::{apply, as_, design, import, reference, submodel, to},
        naming::{identifier, label, render_name},
        structure::end_of_line,
        symbol::{bracket_left, bracket_right, colon, comma, dot, dot_dot, equals, slash},
    },
    util::{InputSpan, Result},
};

/// Parses a complete `design <model>` declaration (for single-decl entry points).
pub fn parse_design_target_complete(input: InputSpan<'_>) -> Result<'_, DeclNode, ParserError> {
    all_consuming(parse_design_target_line).parse(input)
}

/// Parses a declaration at the top level of a model or inside a `section`.
///
/// Design-body shorthand (`id(.<seg>)* = expr`) is enabled when
/// [`Config::allow_design_shorthand`](crate::Config::allow_design_shorthand) is set on the
/// input span's `extra`; the parser flips that flag once the leading `design <model>`
/// line of a `.one` bundle has parsed successfully.
///
/// Tries each declaration parser in sequence:
/// 1. Import declaration (`import path`)
/// 2. Apply declaration (`apply <file> to <target>(.<target>)* [\[ … \]]`)
/// 3. Design target probe (returns error if `design` appears in wrong context)
/// 4. Submodel declaration (`submodel/reference <path> [as alias] [\[ submodels \]]`)
/// 5. Test declaration (`test: condition`)
/// 6. Design parameter shorthand (only when shorthand is enabled)
/// 7. Parameter declaration (parameter definitions)
///
/// The first parser that succeeds determines the declaration type. If a keyword
/// matches but the rest of the declaration is malformed, an error is returned
/// rather than trying subsequent parsers.
pub fn parse(input: InputSpan<'_>) -> Result<'_, DeclNode, ParserError> {
    if let Ok((_, node)) = parse_design_target_line.parse(input.clone()) {
        let err = if input.extra.require_design_header {
            ParserError::design_header_duplicate(node.span().clone())
        } else {
            ParserError::design_header_wrong_file(node.span().clone())
        };
        return Err(nom::Err::Error(err));
    }

    if input.extra.allow_design_shorthand {
        alt((
            import_decl,
            apply_decl,
            submodel_decl,
            test_decl,
            design_parameter_decl.convert_error_to(ParserError::expect_decl),
        ))
        .parse(input)
    } else {
        alt((
            import_decl,
            apply_decl,
            submodel_decl,
            test_decl,
            parameter_decl.convert_error_to(ParserError::expect_decl),
        ))
        .parse(input)
    }
}

/// Parses a declaration (full input).
pub fn parse_complete(input: InputSpan<'_>) -> Result<'_, DeclNode, ParserError> {
    all_consuming(parse).parse(input)
}

/// Parses an import declaration
fn import_decl(input: InputSpan<'_>) -> Result<'_, DeclNode, ParserError> {
    let (rest, import_token) = import.convert_errors().parse(input)?;

    // TODO: allow a path here (ex. `import foo.bar`)
    let (rest, import_path_token) = identifier
        .or_fail_with(ParserError::import_missing_path(
            import_token.lexeme_span.clone(),
        ))
        .parse(rest)?;

    let (rest, end_of_line_token) = end_of_line
        .or_fail_with(ParserError::import_missing_end_of_line(
            import_path_token.lexeme_span.clone(),
        ))
        .parse(rest)?;

    let node_span =
        Span::from_start_and_end(&import_token.lexeme_span, &end_of_line_token.lexeme_span);
    let whitespace_span = end_of_line_token.whitespace_span;

    let import_path_str = Node::<String>::from(import_path_token);

    let import_node = Node::new(
        Import::new(import_path_str),
        node_span.clone(),
        whitespace_span.clone(),
    );

    let decl_node = Node::new(Decl::Import(import_node), node_span, whitespace_span);

    Ok((rest, decl_node))
}

/// Parses a top-level `design [path/to/]<model>` line (`.one` design files and wrong-file probe on `.on`).
pub fn parse_design_target_line(input: InputSpan<'_>) -> Result<'_, DeclNode, ParserError> {
    let (rest, design_token) = design.convert_errors().parse(input)?;

    // Parse optional directory path (e.g., `../models/`)
    let (rest, directory_path) = opt_directory_path.parse(rest)?;

    let (rest, target_token) = identifier
        .or_fail_with(ParserError::design_missing_target(
            design_token.lexeme_span.clone(),
        ))
        .parse(rest)?;

    let (rest, end_of_line_token) = end_of_line
        .or_fail_with(ParserError::import_missing_end_of_line(
            target_token.lexeme_span.clone(),
        ))
        .parse(rest)?;

    let target_node = IdentifierNode::from(target_token);
    let node_span =
        Span::from_start_and_end(&design_token.lexeme_span, &end_of_line_token.lexeme_span);
    let whitespace_span = end_of_line_token.whitespace_span;
    let inner = if directory_path.is_empty() {
        Node::new(
            DesignTarget::new(target_node),
            node_span.clone(),
            whitespace_span.clone(),
        )
    } else {
        Node::new(
            DesignTarget::with_path(directory_path, target_node),
            node_span.clone(),
            whitespace_span.clone(),
        )
    };
    let decl_node = Node::new(Decl::DesignTarget(inner), node_span, whitespace_span);

    Ok((rest, decl_node))
}

/// Parses an `apply [path/to/]<file> to <target>(.<target>)* [ '[' nested_applies ']' ]`
/// declaration (with the `apply` keyword present), terminated by an end-of-line.
fn apply_decl(input: InputSpan<'_>) -> Result<'_, DeclNode, ParserError> {
    let (rest, apply_token) = apply.convert_errors().parse(input)?;
    let (rest, body) = parse_apply_body(rest, Some(apply_token.lexeme_span.clone()))?;

    let body_end_span = body.span();
    let (rest, end_of_line_token) = end_of_line
        .or_fail_with(ParserError::submodel_missing_end_of_line(
            body_end_span.clone(),
        ))
        .parse(rest)?;

    let span_from_kw =
        Span::from_start_and_end(&apply_token.lexeme_span, &end_of_line_token.lexeme_span);
    let whitespace_span = end_of_line_token.whitespace_span;
    Ok((
        rest,
        Node::new(Decl::ApplyDesign(body), span_from_kw, whitespace_span),
    ))
}

/// Parses the body of an apply: `[path/to/]<file> to <target>(.<target>)* [ \[ … \] ]`.
///
/// `apply_kw_span` is `Some` for top-level applies (so error messages can point at the
/// `apply` keyword) and `None` for nested applies inside a `[ … ]` block where the
/// keyword is omitted.
fn parse_apply_body(
    input: InputSpan<'_>,
    apply_kw_span: Option<Span>,
) -> Result<'_, ApplyDesignNode, ParserError> {
    let (rest, directory_path) = opt_directory_path.parse(input)?;

    // For top-level applies (the `apply` keyword has already been consumed), a missing
    // file identifier is a hard failure. For nested applies (called from `many0` inside
    // a `[ … ]` block), it should be a recoverable error so that `many0` stops cleanly
    // at the closing bracket.
    let (rest, file_token) = if let Some(apply_kw_span) = apply_kw_span {
        identifier
            .or_fail_with(ParserError::apply_missing_file(apply_kw_span))
            .parse(rest)?
    } else {
        identifier.convert_errors().parse(rest)?
    };
    let file_node = IdentifierNode::from(file_token);

    let (rest, _to_token) = to
        .or_fail_with(ParserError::apply_missing_target(file_node.span().clone()))
        .parse(rest)?;

    let (rest, first_target) = identifier
        .or_fail_with(ParserError::apply_missing_target(file_node.span().clone()))
        .parse(rest)?;
    let first_target_node = IdentifierNode::from(first_target);
    let mut last_segment_span: Span = first_target_node.span().clone();

    let (rest, more_targets) = many0(|input| {
        let (rest, dot_token) = dot.convert_errors().parse(input)?;
        let (rest, segment_token) = identifier
            .or_fail_with(ParserError::model_path_missing_subcomponent(
                dot_token.lexeme_span,
            ))
            .parse(rest)?;
        Ok((rest, IdentifierNode::from(segment_token)))
    })
    .parse(rest)?;

    if let Some(last) = more_targets.last() {
        last_segment_span = last.span().clone();
    }

    let target: Vec<IdentifierNode> = std::iter::once(first_target_node)
        .chain(more_targets)
        .collect();

    // Optional `[ nested_applies ]` block.
    let (rest, nested_block) = opt(|input| {
        let (rest, bracket_left_token) = bracket_left.convert_errors().parse(input)?;
        let (rest, _) = opt(end_of_line).convert_errors().parse(rest)?;
        let (rest, items) = many0(nested_apply_item).parse(rest)?;
        let (rest, bracket_right_token) = bracket_right
            .or_fail_with(ParserError::unclosed_bracket(
                bracket_left_token.lexeme_span.clone(),
            ))
            .parse(rest)?;
        Ok((rest, (bracket_left_token, bracket_right_token, items)))
    })
    .parse(rest)?;

    let (body_end_span, nested_applies) = match nested_block {
        Some((_, bracket_right_token, items)) => (bracket_right_token.lexeme_span, items),
        None => (last_segment_span, Vec::new()),
    };

    let body_start_span = if directory_path.is_empty() {
        file_node.span()
    } else {
        directory_path[0].span()
    };
    let node_span = Span::from_start_and_end(body_start_span, &body_end_span);
    let whitespace_span = body_end_span;

    let inner = ApplyDesign::new(directory_path, file_node, target, nested_applies);
    Ok((rest, Node::new(inner, node_span, whitespace_span)))
}

/// Parses a single nested apply inside a `[ … ]` block. No `apply` keyword. Items are
/// separated by either a comma or an end-of-line; both are optional after the last item.
fn nested_apply_item(input: InputSpan<'_>) -> Result<'_, ApplyDesignNode, ParserError> {
    let (rest, body) = parse_apply_body(input, None)?;
    let (rest, _comma_or_eol) = opt(alt((
        comma.map(|_| ()).convert_errors(),
        end_of_line.map(|_| ()).convert_errors(),
    )))
    .parse(rest)?;
    let (rest, _) = opt(end_of_line).convert_errors().parse(rest)?;
    Ok((rest, body))
}

/// Parses a parameter line in a design file (after `design`).
///
/// Handles two forms:
/// - Shorthand: `[$] [*[*]] id[.segment] = value` — no label, `instance_path` allowed.
/// - Full:      `[$] [*[*]] Label text: id = value` — explicit label, no instance path.
///
/// The `$` performance marker and trace-level prefix only take effect when the
/// parameter is a new addition (not an override of an existing parameter).
fn design_parameter_decl(input: InputSpan<'_>) -> Result<'_, DeclNode, ParserError> {
    // Optional output-parameter marker (`$`) and trace level (`*` / `**`),
    // reusing the same parsers as regular ParameterDecl. These only have an
    // effect when the line introduces a new parameter (i.e. the name is not
    // present on the design target); they are silently ignored for overrides.
    let (rest, performance_marker_node) = opt(performance_marker).parse(input)?;
    let (rest, trace_level_node) = opt(trace_level).parse(rest)?;

    // Try to parse an optional `Label text:` prefix. This distinguishes the full
    // parameter form (`Label: id = value`) from the shorthand form (`id = value`).
    // The label parser is greedy but backtracking is safe: if `label` matches but
    // the following `:` is absent (e.g. input is `id = expr`), `opt` resets.
    let (rest, label_node) = opt(|input| {
        let (rest, label_token) = label.convert_errors().parse(input)?;
        let label_node = ParameterLabelNode::from(label_token);
        let (rest, _) = colon.convert_errors().parse(rest)?;
        Ok((rest, label_node))
    })
    .parse(rest)?;

    // Optional LaTeX render-name (`{...}`) only valid in the full form (when a label
    // is present). In shorthand form it is skipped entirely so that piecewise `{` is
    // not mis-parsed as a render-name opener.
    let (rest, render_name_node): (_, Option<RenderNameNode>) = if label_node.is_some() {
        let (rest, token) = opt(render_name.convert_errors()).parse(rest)?;
        let node = token.map(|t| {
            let content = t.lexeme_str[1..t.lexeme_str.len() - 1].to_string();
            t.into_node_with_value(RenderName::new(content))
        });
        (rest, node)
    } else {
        (rest, None)
    };

    let (rest, ident_token) = identifier.convert_errors().parse(rest)?;
    let ident_node = IdentifierNode::from(ident_token);

    // Instance-path scoping (`mass.sat = …`) is only meaningful for shorthand
    // overrides; labeled additions target the design directly.
    let (rest, instance_path) = if label_node.is_none() {
        opt(|input| {
            let (rest, dot_token) = dot.convert_errors().parse(input)?;
            let (rest, segment_token) = identifier
                .or_fail_with(ParserError::parameter_missing_instance_path_segment(
                    dot_token.lexeme_span,
                ))
                .parse(rest)?;
            Ok((rest, IdentifierNode::from(segment_token)))
        })
        .parse(rest)?
    } else {
        (rest, None)
    };

    let (rest, equals_token) = equals.convert_errors().parse(rest)?;

    let (rest, value_node) = parse_parameter_value
        .or_fail_with(ParserError::parameter_missing_value(
            equals_token.lexeme_span,
        ))
        .parse(rest)?;

    let (rest, linebreak_token) = end_of_line
        .or_fail_with(ParserError::parameter_missing_end_of_line(
            value_node.span().clone(),
        ))
        .parse(rest)?;

    let (rest, note_node) = opt(parse_note).parse(rest)?;

    let param_start_span = match (&performance_marker_node, &trace_level_node) {
        (Some(m), _) => m.span(),
        (None, Some(t)) => t.span(),
        (None, None) => label_node
            .as_ref()
            .map_or_else(|| ident_node.span(), Node::span),
    };
    let (param_end_span, param_whitespace_span) = note_node.as_ref().map_or(
        (linebreak_token.lexeme_span, linebreak_token.whitespace_span),
        |note_node| {
            (
                note_node.span().clone(),
                note_node.whitespace_span().clone(),
            )
        },
    );

    let param_span = Span::from_start_and_end(param_start_span, &param_end_span);
    let inner = DesignParameter::new(
        ident_node,
        instance_path,
        value_node,
        performance_marker_node,
        trace_level_node,
        note_node,
        label_node,
        render_name_node,
    );
    let inner_node = Node::new(inner, param_span.clone(), param_whitespace_span.clone());
    let decl_node = Node::new(
        Decl::DesignParameter(inner_node),
        param_span,
        param_whitespace_span,
    );

    Ok((rest, decl_node))
}

/// Parses a submodel declaration (`submodel` or `reference` plus path, optional alias,
/// and optional `[ submodels ]` extraction block).
fn submodel_decl(input: InputSpan<'_>) -> Result<'_, DeclNode, ParserError> {
    let reference_keyword = |input| {
        let (rest, tok) = reference.convert_errors().parse(input)?;
        Ok((
            rest,
            (ModelKind::Reference, SubmodelKeyword::Reference, tok),
        ))
    };

    let submodel_keyword = |input| {
        let (rest, tok) = submodel.convert_errors().parse(input)?;
        Ok((rest, (ModelKind::Submodel, SubmodelKeyword::Submodel, tok)))
    };

    let (rest, (model_kind, keyword, keyword_token)) =
        alt((reference_keyword, submodel_keyword)).parse(input)?;

    let (rest, directory_path) = opt_directory_path.parse(rest)?;

    let (rest, model_info) = model_info_simple
        .or_fail_with(ParserError::submodel_missing_model_info(
            keyword_token.lexeme_span,
            keyword,
        ))
        .parse(rest)?;

    // Bracketed extraction block: `submodel sat as s [ a as alpha, b ]`. The bracket
    // itself denotes extraction; there is no introducing keyword.
    let (rest, submodel_list) = opt(submodel_list).parse(rest)?;

    let final_span: Span = submodel_list
        .as_ref()
        .map_or_else(|| model_info.span().clone(), |n| n.span().clone());

    let (rest, end_of_line_token) = end_of_line
        .or_fail_with(ParserError::submodel_missing_end_of_line(
            final_span.clone(),
        ))
        .parse(rest)?;

    let submodel_node = Node::new(
        SubmodelDecl::new(directory_path, model_info, submodel_list, model_kind),
        final_span.clone(),
        end_of_line_token.whitespace_span.clone(),
    );

    let decl_node = Node::new(
        Decl::Submodel(submodel_node),
        final_span,
        end_of_line_token.whitespace_span,
    );

    Ok((rest, decl_node))
}

/// Parses a directory path in a model path
fn opt_directory_path(input: InputSpan<'_>) -> Result<'_, Vec<DirectoryNode>, ParserError> {
    many0(|input| {
        let (rest, directory_name) = directory_name.parse(input)?;
        let (rest, _slash_token) = slash.convert_errors().parse(rest)?;
        Ok((rest, directory_name))
    })
    .parse(input)
}

/// Parses a directory name in a model path
fn directory_name(input: InputSpan<'_>) -> Result<'_, DirectoryNode, ParserError> {
    let directory_name = |input| {
        let (rest, directory_name_token) = identifier.convert_errors().parse(input)?;
        let directory_name = DirectoryNode::from(directory_name_token);
        Ok((rest, directory_name))
    };

    let current_directory = |input| {
        let (rest, dot_token) = dot.convert_errors().parse(input)?;
        let current_directory = dot_token.into_node_with_value(Directory::current());
        Ok((rest, current_directory))
    };

    let parent_directory = |input| {
        let (rest, dot_dot_token) = dot_dot.convert_errors().parse(input)?;
        let parent_directory = dot_dot_token.into_node_with_value(Directory::parent());
        Ok((rest, parent_directory))
    };

    alt((directory_name, current_directory, parent_directory)).parse(input)
}

/// Parses a model info without subcomponents (for the main model in `submodel`/`reference`).
fn model_info_simple(input: InputSpan<'_>) -> Result<'_, ModelInfoNode, ParserError> {
    let (rest, top_component_token) = identifier.convert_errors().parse(input)?;
    let top_component_node = IdentifierNode::from(top_component_token);

    let (rest, alias) = opt(as_alias).parse(rest)?;

    let (final_span, whitespace_span): (Span, Span) = alias.as_ref().map_or_else(
        || {
            (
                top_component_node.span().clone(),
                top_component_node.whitespace_span().clone(),
            )
        },
        |a| (a.span().clone(), a.whitespace_span().clone()),
    );

    let model_info_span = Span::from_start_and_end(top_component_node.span(), &final_span);
    let model_info = ModelInfo::new(top_component_node, vec![], alias);

    Ok((
        rest,
        Node::new(model_info, model_info_span, whitespace_span),
    ))
}

/// Parses a model info with optional subcomponents (for submodels inside an extraction block).
pub fn model_info(input: InputSpan<'_>) -> Result<'_, ModelInfoNode, ParserError> {
    let (rest, top_component_token) = identifier.convert_errors().parse(input)?;
    let top_component_node = IdentifierNode::from(top_component_token);

    let (rest, subcomponents) = opt_subcomponents.parse(rest)?;
    let (rest, alias) = opt(as_alias).parse(rest)?;

    let (final_span, whitespace_span): (Span, Span) = match (subcomponents.last(), &alias) {
        (_, Some(alias)) => (alias.span().clone(), alias.whitespace_span().clone()),
        (Some(subcomponent), None) => (
            subcomponent.span().clone(),
            subcomponent.whitespace_span().clone(),
        ),
        (None, None) => (
            top_component_node.span().clone(),
            top_component_node.whitespace_span().clone(),
        ),
    };

    let model_info_span = Span::from_start_and_end(top_component_node.span(), &final_span);
    let model_info = ModelInfo::new(top_component_node, subcomponents, alias);

    Ok((
        rest,
        Node::new(model_info, model_info_span, whitespace_span),
    ))
}

fn opt_subcomponents(input: InputSpan<'_>) -> Result<'_, Vec<IdentifierNode>, ParserError> {
    many0(|input| {
        let (rest, dot_token) = dot.convert_errors().parse(input)?;

        let (rest, subcomponent_token) = identifier
            .or_fail_with(ParserError::model_path_missing_subcomponent(
                dot_token.lexeme_span,
            ))
            .parse(rest)?;

        let subcomponent_node = IdentifierNode::from(subcomponent_token);

        Ok((rest, subcomponent_node))
    })
    .parse(input)
}

/// Parses an alias identifier after an `as` keyword.
fn as_alias(input: InputSpan<'_>) -> Result<'_, IdentifierNode, ParserError> {
    let (rest, as_token) = as_.convert_errors().parse(input)?;

    let (rest, alias_token) = identifier
        .or_fail_with(ParserError::as_missing_alias(as_token.lexeme_span))
        .parse(rest)?;

    let alias_node = IdentifierNode::from(alias_token);

    Ok((rest, alias_node))
}

/// Parses a bracketed extraction block (`[ a, b as beta, c.x as cx ]`) of submodels.
///
/// Brackets are mandatory and there is no introducing keyword: the `[ … ]` itself denotes the
/// extraction list.
fn submodel_list(input: InputSpan<'_>) -> Result<'_, SubmodelListNode, ParserError> {
    let (rest, bracket_left_token) = bracket_left.convert_errors().parse(input)?;

    let (rest, _optional_end_of_line_token) = opt(end_of_line).convert_errors().parse(rest)?;

    let (rest, submodel_list) = opt(|input| {
        let (rest, first_submodel) = model_info.parse(input)?;

        let (rest, rest_submodels) = many0(|input| {
            let (rest, _comma_token) = comma.convert_errors().parse(input)?;
            let (rest, _optional_end_of_line_token) =
                opt(end_of_line).convert_errors().parse(rest)?;
            // Normally, this `submodel` parsing would have `or_fail_with`
            // since we have found a comma token. However, the comma may be
            // the optional trailing comma, so we don't fail here.
            let (rest, submodel) = model_info.parse(rest)?;
            Ok((rest, submodel))
        })
        .parse(rest)?;

        let (rest, _optional_trailing_comma_token) = opt(comma).convert_errors().parse(rest)?;
        let (rest, _optional_end_of_line_token) = opt(end_of_line).convert_errors().parse(rest)?;

        let mut submodels = rest_submodels;
        submodels.insert(0, first_submodel);
        Ok((rest, submodels))
    })
    .parse(rest)?;

    let (rest, bracket_right_token) = bracket_right
        .or_fail_with(ParserError::unclosed_bracket(
            bracket_left_token.lexeme_span.clone(),
        ))
        .parse(rest)?;

    let submodel_list = SubmodelList::new(submodel_list.unwrap_or_default());
    let submodel_list_span = Span::from_start_and_end(
        &bracket_left_token.lexeme_span,
        &bracket_right_token.lexeme_span,
    );
    let submodel_list_whitespace_span = bracket_right_token.whitespace_span;

    let submodel_list_node = Node::new(
        submodel_list,
        submodel_list_span,
        submodel_list_whitespace_span,
    );

    Ok((rest, submodel_list_node))
}

/// Parses a parameter declaration by delegating to the parameter parser.
fn parameter_decl(input: InputSpan<'_>) -> Result<'_, DeclNode, ParserError> {
    let (rest, parameter) = parse_parameter.parse(input)?;

    let parameter_span = parameter.span().clone();
    let parameter_whitespace_span = parameter.whitespace_span().clone();
    let decl_node = Node::new(
        Decl::Parameter(parameter),
        parameter_span,
        parameter_whitespace_span,
    );

    Ok((rest, decl_node))
}

/// Parses a test declaration by delegating to the test parser.
fn test_decl(input: InputSpan<'_>) -> Result<'_, DeclNode, ParserError> {
    let (rest, test) = parse_test.parse(input)?;

    let span = test.span().clone();
    let whitespace_span = test.whitespace_span().clone();
    let decl_node = Node::new(Decl::Test(test), span, whitespace_span);

    Ok((rest, decl_node))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;

    mod success {
        use super::*;

        #[test]
        fn import_decl() {
            let input = InputSpan::new_extra("import foo\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::Import(ref import_node) = *decl else {
                panic!("Expected import declaration");
            };

            assert_eq!(import_node.path().as_str(), "foo");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn reference_decl() {
            let input = InputSpan::new_extra("reference foo\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::Submodel(ref submodel_node) = *decl else {
                panic!("Expected submodel declaration");
            };

            assert_eq!(submodel_node.model_info().get_alias().as_str(), "foo");
            assert_eq!(submodel_node.model_kind(), ModelKind::Reference);
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn submodel_decl_simple_without_alias() {
            let input = InputSpan::new_extra("submodel foo\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::Submodel(ref submodel_node) = *decl else {
                panic!("Expected submodel declaration");
            };

            let model_info = submodel_node.model_info();
            assert_eq!(model_info.top_component().as_str(), "foo");
            assert_eq!(model_info.subcomponents().len(), 0);
            assert_eq!(model_info.get_alias().as_str(), "foo");
            assert_eq!(submodel_node.model_kind(), ModelKind::Submodel);
            assert!(submodel_node.imported_submodels().is_none());
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn submodel_decl_with_alias() {
            let input = InputSpan::new_extra("submodel foo as bar\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::Submodel(ref submodel_node) = *decl else {
                panic!("Expected submodel declaration");
            };

            let model_info = submodel_node.model_info();
            assert_eq!(model_info.top_component().as_str(), "foo");
            assert_eq!(model_info.get_alias().as_str(), "bar");
            assert_eq!(submodel_node.model_kind(), ModelKind::Submodel);
            assert!(submodel_node.imported_submodels().is_none());
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn submodel_with_extraction_brackets() {
            let input = InputSpan::new_extra("submodel foo as f [bar, baz]\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::Submodel(ref submodel_node) = *decl else {
                panic!("Expected submodel declaration");
            };
            assert_eq!(submodel_node.model_info().get_alias().as_str(), "f");
            let submodels = submodel_node
                .imported_submodels()
                .expect("should have submodels");
            assert_eq!(submodels.len(), 2);
            assert_eq!(submodels[0].get_alias().as_str(), "bar");
            assert_eq!(submodels[1].get_alias().as_str(), "baz");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn submodel_with_single_extracted_submodel_and_alias() {
            let input = InputSpan::new_extra("submodel foo [bar as baz]\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::Submodel(ref submodel_node) = *decl else {
                panic!("Expected submodel declaration");
            };

            let model_info = submodel_node.model_info();
            assert_eq!(model_info.top_component().as_str(), "foo");
            assert_eq!(model_info.get_alias().as_str(), "foo");
            assert_eq!(submodel_node.model_kind(), ModelKind::Submodel);

            let submodels = submodel_node
                .imported_submodels()
                .expect("should have submodels");
            assert_eq!(submodels.len(), 1);
            assert_eq!(submodels[0].top_component().as_str(), "bar");
            assert_eq!(submodels[0].get_alias().as_str(), "baz");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn submodel_with_single_extracted_submodel_no_alias() {
            let input = InputSpan::new_extra("submodel foo [bar]\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::Submodel(ref submodel_node) = *decl else {
                panic!("Expected submodel declaration");
            };

            let submodels = submodel_node
                .imported_submodels()
                .expect("should have submodels");
            assert_eq!(submodels.len(), 1);
            assert_eq!(submodels[0].top_component().as_str(), "bar");
            assert_eq!(submodels[0].get_alias().as_str(), "bar");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn parse_complete_submodel_success() {
            let input = InputSpan::new_extra("submodel foo [bar as baz]\n", Config::default());
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");

            let Decl::Submodel(ref submodel_node) = *decl else {
                panic!("Expected submodel declaration");
            };

            assert_eq!(submodel_node.model_info().top_component().as_str(), "foo");
            let submodels = submodel_node
                .imported_submodels()
                .expect("should have submodels");
            assert_eq!(submodels.len(), 1);
            assert_eq!(submodels[0].get_alias().as_str(), "baz");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn submodel_with_single_directory() {
            let input =
                InputSpan::new_extra("submodel utils/math as calculator\n", Config::default());
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");

            let Decl::Submodel(ref submodel_node) = *decl else {
                panic!("Expected submodel declaration");
            };

            let model_info = submodel_node.model_info();
            assert_eq!(model_info.top_component().as_str(), "math");
            assert_eq!(model_info.get_alias().as_str(), "calculator");

            assert_eq!(submodel_node.directory_path().len(), 1);
            assert_eq!(submodel_node.directory_path()[0].as_str(), "utils");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn submodel_with_single_directory_without_alias() {
            let input = InputSpan::new_extra("submodel utils/math\n", Config::default());
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");

            let Decl::Submodel(ref submodel_node) = *decl else {
                panic!("Expected submodel declaration");
            };

            let model_info = submodel_node.model_info();
            assert_eq!(model_info.top_component().as_str(), "math");
            assert_eq!(model_info.get_alias().as_str(), "math");

            assert_eq!(submodel_node.directory_path().len(), 1);
            assert_eq!(submodel_node.directory_path()[0].as_str(), "utils");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn submodel_with_multiple_directories() {
            let input = InputSpan::new_extra(
                "submodel models/physics/mechanics as dynamics\n",
                Config::default(),
            );
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");

            let Decl::Submodel(ref submodel_node) = *decl else {
                panic!("Expected submodel declaration");
            };

            let model_info = submodel_node.model_info();
            assert_eq!(model_info.top_component().as_str(), "mechanics");
            assert_eq!(model_info.get_alias().as_str(), "dynamics");

            assert_eq!(submodel_node.directory_path().len(), 2);
            assert_eq!(submodel_node.directory_path()[0].as_str(), "models");
            assert_eq!(submodel_node.directory_path()[1].as_str(), "physics");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn submodel_with_directory_and_extracted_submodels() {
            let input = InputSpan::new_extra(
                "submodel utils/math [trigonometry as trig]\n",
                Config::default(),
            );
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");

            let Decl::Submodel(ref submodel_node) = *decl else {
                panic!("Expected submodel declaration");
            };

            let model_info = submodel_node.model_info();
            assert_eq!(model_info.top_component().as_str(), "math");
            assert_eq!(model_info.get_alias().as_str(), "math");

            let submodels = submodel_node
                .imported_submodels()
                .expect("should have submodels");
            assert_eq!(submodels.len(), 1);
            assert_eq!(submodels[0].top_component().as_str(), "trigonometry");
            assert_eq!(submodels[0].get_alias().as_str(), "trig");

            assert_eq!(submodel_node.directory_path().len(), 1);
            assert_eq!(submodel_node.directory_path()[0].as_str(), "utils");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn submodel_with_current_directory() {
            let input =
                InputSpan::new_extra("submodel ./local_model as local\n", Config::default());
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");

            let Decl::Submodel(ref submodel_node) = *decl else {
                panic!("Expected submodel declaration");
            };

            let model_info = submodel_node.model_info();
            assert_eq!(model_info.top_component().as_str(), "local_model");
            assert_eq!(model_info.get_alias().as_str(), "local");

            assert_eq!(submodel_node.directory_path().len(), 1);
            assert_eq!(submodel_node.directory_path()[0].as_str(), ".");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn submodel_with_parent_directory() {
            let input =
                InputSpan::new_extra("submodel ../parent_model as parent\n", Config::default());
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");

            let Decl::Submodel(ref submodel_node) = *decl else {
                panic!("Expected submodel declaration");
            };

            let model_info = submodel_node.model_info();
            assert_eq!(model_info.top_component().as_str(), "parent_model");
            assert_eq!(model_info.get_alias().as_str(), "parent");

            assert_eq!(submodel_node.directory_path().len(), 1);
            assert_eq!(submodel_node.directory_path()[0].as_str(), "..");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn submodel_with_mixed_directory_types() {
            let input = InputSpan::new_extra(
                "submodel ../shared/./utils/math as shared_math\n",
                Config::default(),
            );
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");

            let Decl::Submodel(ref submodel_node) = *decl else {
                panic!("Expected submodel declaration");
            };

            let model_info = submodel_node.model_info();
            assert_eq!(model_info.top_component().as_str(), "math");
            assert_eq!(model_info.get_alias().as_str(), "shared_math");

            assert_eq!(submodel_node.directory_path().len(), 4);
            assert_eq!(submodel_node.directory_path()[0].as_str(), "..");
            assert_eq!(submodel_node.directory_path()[1].as_str(), "shared");
            assert_eq!(submodel_node.directory_path()[2].as_str(), ".");
            assert_eq!(submodel_node.directory_path()[3].as_str(), "utils");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn submodel_with_complex_path_and_subcomponents() {
            let input = InputSpan::new_extra(
                "submodel models/physics/mechanics [rotational.dynamics as rotation]\n",
                Config::default(),
            );
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");

            let Decl::Submodel(ref submodel_node) = *decl else {
                panic!("Expected submodel declaration");
            };

            let model_info = submodel_node.model_info();
            assert_eq!(model_info.top_component().as_str(), "mechanics");
            assert_eq!(model_info.get_alias().as_str(), "mechanics");

            let submodels = submodel_node
                .imported_submodels()
                .expect("should have submodels");
            assert_eq!(submodels.len(), 1);
            assert_eq!(submodels[0].top_component().as_str(), "rotational");
            assert_eq!(submodels[0].subcomponents().len(), 1);
            assert_eq!(submodels[0].subcomponents()[0].as_str(), "dynamics");
            assert_eq!(submodels[0].get_alias().as_str(), "rotation");

            assert_eq!(submodel_node.directory_path().len(), 2);
            assert_eq!(submodel_node.directory_path()[0].as_str(), "models");
            assert_eq!(submodel_node.directory_path()[1].as_str(), "physics");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn directory_name_parsing() {
            let input = InputSpan::new_extra("..", Config::default());
            let (rest, dir) = directory_name(input).expect("should parse parent directory");
            assert_eq!(dir.as_str(), "..");
            assert_eq!(rest.fragment(), &"");

            let input = InputSpan::new_extra(".", Config::default());
            let (rest, dir) = directory_name(input).expect("should parse current directory");
            assert_eq!(dir.as_str(), ".");
            assert_eq!(rest.fragment(), &"");

            let input = InputSpan::new_extra("foo", Config::default());
            let (rest, dir) = directory_name(input).expect("should parse regular directory name");
            assert_eq!(dir.as_str(), "foo");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn mixed_directory_path_parsing() {
            let input = InputSpan::new_extra("../shared/./utils/", Config::default());
            let (_rest, directory_path) =
                opt_directory_path(input).expect("should parse mixed directory path");

            assert_eq!(directory_path.len(), 4);
            assert_eq!(*directory_path[0], Directory::Parent);
            assert_eq!(*directory_path[1], Directory::Name("shared".to_string()));
            assert_eq!(*directory_path[2], Directory::Current);
            assert_eq!(*directory_path[3], Directory::Name("utils".to_string()));
        }

        #[test]
        fn submodel_decl_with_extracted_submodel_with_subcomponents() {
            let input = InputSpan::new_extra("submodel foo [bar.qux]\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::Submodel(ref submodel_node) = *decl else {
                panic!("Expected submodel declaration");
            };

            let submodels = submodel_node
                .imported_submodels()
                .expect("should have submodels");
            assert_eq!(submodels.len(), 1);
            let submodel = &submodels[0];
            assert_eq!(submodel.top_component().as_str(), "bar");
            assert_eq!(submodel.subcomponents().len(), 1);
            assert_eq!(submodel.subcomponents()[0].as_str(), "qux");
            assert_eq!(submodel.get_alias().as_str(), "qux");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn submodel_decl_with_multiple_extracted_submodels() {
            let input = InputSpan::new_extra("submodel foo [bar, qux]\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::Submodel(ref submodel_node) = *decl else {
                panic!("Expected submodel declaration");
            };

            let submodels = submodel_node
                .imported_submodels()
                .expect("should have submodels");
            assert_eq!(submodels.len(), 2);
            assert_eq!(submodels[0].top_component().as_str(), "bar");
            assert_eq!(submodels[0].get_alias().as_str(), "bar");
            assert_eq!(submodels[1].top_component().as_str(), "qux");
            assert_eq!(submodels[1].get_alias().as_str(), "qux");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn submodel_decl_with_multiple_extracted_submodels_with_aliases() {
            let input = InputSpan::new_extra(
                "submodel foo [bar as baz, qux as quux]\n",
                Config::default(),
            );
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::Submodel(ref submodel_node) = *decl else {
                panic!("Expected submodel declaration");
            };

            let submodels = submodel_node
                .imported_submodels()
                .expect("should have submodels");
            assert_eq!(submodels.len(), 2);
            assert_eq!(submodels[0].top_component().as_str(), "bar");
            assert_eq!(submodels[0].get_alias().as_str(), "baz");
            assert_eq!(submodels[1].top_component().as_str(), "qux");
            assert_eq!(submodels[1].get_alias().as_str(), "quux");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn submodel_decl_with_multiple_submodels_and_subcomponents() {
            let input =
                InputSpan::new_extra("submodel foo [bar.qux, baz.quux.quuz]\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::Submodel(ref submodel_node) = *decl else {
                panic!("Expected submodel declaration");
            };

            let submodels = submodel_node
                .imported_submodels()
                .expect("should have submodels");
            assert_eq!(submodels.len(), 2);

            let s1 = &submodels[0];
            assert_eq!(s1.top_component().as_str(), "bar");
            assert_eq!(s1.subcomponents().len(), 1);
            assert_eq!(s1.subcomponents()[0].as_str(), "qux");
            assert_eq!(s1.get_alias().as_str(), "qux");

            let s2 = &submodels[1];
            assert_eq!(s2.top_component().as_str(), "baz");
            assert_eq!(s2.subcomponents().len(), 2);
            assert_eq!(s2.subcomponents()[0].as_str(), "quux");
            assert_eq!(s2.subcomponents()[1].as_str(), "quuz");
            assert_eq!(s2.get_alias().as_str(), "quuz");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn submodel_decl_with_trailing_comma() {
            let input = InputSpan::new_extra("submodel foo [bar, qux,]\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::Submodel(ref submodel_node) = *decl else {
                panic!("Expected submodel declaration");
            };

            let submodels = submodel_node
                .imported_submodels()
                .expect("should have submodels");
            assert_eq!(submodels.len(), 2);
            assert_eq!(submodels[0].get_alias().as_str(), "bar");
            assert_eq!(submodels[1].get_alias().as_str(), "qux");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn submodel_decl_with_empty_submodel_list() {
            let input = InputSpan::new_extra("submodel foo []\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::Submodel(ref submodel_node) = *decl else {
                panic!("Expected submodel declaration");
            };

            let submodels = submodel_node
                .imported_submodels()
                .expect("should have submodels");
            assert_eq!(submodels.len(), 0);

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn submodel_decl_with_model_alias_and_extracted_submodels() {
            let input = InputSpan::new_extra("submodel foo as bar [qux, baz]\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::Submodel(ref submodel_node) = *decl else {
                panic!("Expected submodel declaration");
            };

            assert_eq!(submodel_node.model_info().top_component().as_str(), "foo");
            assert_eq!(submodel_node.model_info().get_alias().as_str(), "bar");

            let submodels = submodel_node
                .imported_submodels()
                .expect("should have submodels");
            assert_eq!(submodels.len(), 2);
            assert_eq!(submodels[0].get_alias().as_str(), "qux");
            assert_eq!(submodels[1].get_alias().as_str(), "baz");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn submodel_decl_with_complex_path_and_extracted_submodels() {
            let input = InputSpan::new_extra(
                "submodel utils/math [trigonometry as trig, sin, cos as cosine]\n",
                Config::default(),
            );
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");

            let Decl::Submodel(ref submodel_node) = *decl else {
                panic!("Expected submodel declaration");
            };

            assert_eq!(submodel_node.directory_path().len(), 1);
            assert_eq!(submodel_node.directory_path()[0].as_str(), "utils");

            let submodels = submodel_node
                .imported_submodels()
                .expect("should have submodels");
            assert_eq!(submodels.len(), 3);
            assert_eq!(submodels[0].top_component().as_str(), "trigonometry");
            assert_eq!(submodels[0].get_alias().as_str(), "trig");
            assert_eq!(submodels[1].top_component().as_str(), "sin");
            assert_eq!(submodels[1].get_alias().as_str(), "sin");
            assert_eq!(submodels[2].top_component().as_str(), "cos");
            assert_eq!(submodels[2].get_alias().as_str(), "cosine");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn submodel_decl_with_extracted_submodels_and_newlines() {
            let input = InputSpan::new_extra("submodel foo [\nbar,\nqux\n]\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::Submodel(ref submodel_node) = *decl else {
                panic!("Expected submodel declaration");
            };

            let submodels = submodel_node
                .imported_submodels()
                .expect("should have submodels");
            assert_eq!(submodels.len(), 2);
            assert_eq!(submodels[0].get_alias().as_str(), "bar");
            assert_eq!(submodels[1].get_alias().as_str(), "qux");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn apply_decl_simple() {
            let input = InputSpan::new_extra("apply uhf to U\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::ApplyDesign(ref apply_node) = *decl else {
                panic!("Expected apply declaration");
            };
            assert_eq!(apply_node.design_file().as_str(), "uhf");
            assert_eq!(apply_node.target().len(), 1);
            assert_eq!(apply_node.target()[0].as_str(), "U");
            assert!(apply_node.nested_applies().is_empty());
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn apply_decl_with_dotted_target_and_nested_block() {
            let input = InputSpan::new_extra(
                "apply uhf to sc.U [\n  alibaba to a,\n]\n",
                Config::default(),
            );
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::ApplyDesign(ref apply_node) = *decl else {
                panic!("Expected apply declaration");
            };
            assert_eq!(apply_node.design_file().as_str(), "uhf");
            assert_eq!(apply_node.target().len(), 2);
            assert_eq!(apply_node.target()[0].as_str(), "sc");
            assert_eq!(apply_node.target()[1].as_str(), "U");
            assert_eq!(apply_node.nested_applies().len(), 1);
            let nested = &apply_node.nested_applies()[0];
            assert_eq!(nested.design_file().as_str(), "alibaba");
            assert_eq!(nested.target().len(), 1);
            assert_eq!(nested.target()[0].as_str(), "a");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn design_parameter_dotted_path() {
            let config = Config {
                allow_design_shorthand: true,
                ..Config::default()
            };
            let input = InputSpan::new_extra("thrust.main_engine = 2000\n", config);
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::DesignParameter(ref param_node) = *decl else {
                panic!("Expected design parameter");
            };
            assert_eq!(param_node.ident().as_str(), "thrust");
            let seg = param_node
                .instance_path()
                .expect("should have one instance segment");
            assert_eq!(seg.as_str(), "main_engine");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn design_parameter_full_with_render_name_no_space_after_colon() {
            // Render name immediately adjacent to colon (no space).
            let config = Config {
                allow_design_shorthand: true,
                ..Config::default()
            };
            let input =
                InputSpan::new_extra("Surface area:{A_{\\mathrm{s}}} A = 4 * pi * R^2\n", config);
            let (_, decl) = parse(input).expect("should parse with no space after colon");
            let Decl::DesignParameter(ref param_node) = *decl else {
                panic!("Expected design parameter");
            };
            assert_eq!(
                param_node
                    .render_name()
                    .expect("should have render name")
                    .as_str(),
                "A_{\\mathrm{s}}"
            );
            assert_eq!(param_node.ident().as_str(), "A");
        }

        #[test]
        fn design_parameter_full_with_render_name() {
            let config = Config {
                allow_design_shorthand: true,
                ..Config::default()
            };
            let input =
                InputSpan::new_extra("Surface area: {A_{\\mathrm{s}}} A = 4 * pi * R^2\n", config);
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::DesignParameter(ref param_node) = *decl else {
                panic!("Expected design parameter");
            };
            assert_eq!(
                param_node.label().expect("should have label").as_str(),
                "Surface area"
            );
            assert_eq!(
                param_node
                    .render_name()
                    .expect("should have render name")
                    .as_str(),
                "A_{\\mathrm{s}}"
            );
            assert_eq!(param_node.ident().as_str(), "A");
            assert!(param_node.instance_path().is_none());
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn design_parameter_full_without_render_name() {
            let config = Config {
                allow_design_shorthand: true,
                ..Config::default()
            };
            let input = InputSpan::new_extra("Surface area: A = 4 * pi * R^2\n", config);
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::DesignParameter(ref param_node) = *decl else {
                panic!("Expected design parameter");
            };
            assert_eq!(
                param_node.label().expect("should have label").as_str(),
                "Surface area"
            );
            assert!(param_node.render_name().is_none());
            assert_eq!(param_node.ident().as_str(), "A");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn design_parameter_shorthand_no_render_name() {
            let config = Config {
                allow_design_shorthand: true,
                ..Config::default()
            };
            let input = InputSpan::new_extra("mass = 5\n", config);
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::DesignParameter(ref param_node) = *decl else {
                panic!("Expected design parameter");
            };
            assert!(param_node.label().is_none());
            assert!(param_node.render_name().is_none());
            assert_eq!(param_node.ident().as_str(), "mass");
            assert_eq!(rest.fragment(), &"");
        }
    }

    mod error {
        use super::*;
        use crate::error::reason::{
            DeclKind, ExpectKind, ImportKind, IncompleteKind, ParserErrorReason, SubmodelKeyword,
            SubmodelKind,
        };
        use crate::token::error::{ExpectKind as TokenExpectKind, TokenErrorKind};

        /// Asserts that `parse(input_str)` returns `Err(Failure(...))` with the
        /// given `IncompleteKind` and cause span.
        fn assert_failure(
            input_str: &str,
            error_offset: usize,
            expected_kind: IncompleteKind,
            cause_start: usize,
            cause_end: usize,
        ) {
            let input = InputSpan::new_extra(input_str, Config::default());
            let error = match parse(input) {
                Err(nom::Err::Failure(e) | nom::Err::Error(e)) => e,
                Ok(_) => panic!("Expected error for {input_str:?}"),
                Err(e) => panic!("Unexpected nom result for {input_str:?}: {e:?}"),
            };
            assert_eq!(error.error_offset, error_offset, "offset for {input_str:?}");
            let ParserErrorReason::Incomplete { kind, cause } = error.reason else {
                panic!(
                    "Expected Incomplete for {input_str:?}, got {:?}",
                    error.reason
                );
            };
            assert_eq!(kind, expected_kind, "kind for {input_str:?}");
            assert_eq!(
                cause.start().offset,
                cause_start,
                "cause_start for {input_str:?}"
            );
            assert_eq!(cause.end().offset, cause_end, "cause_end for {input_str:?}");
        }

        #[test]
        fn expect_decl_errors() {
            let cases: &[&str] = &[
                "",
                "foo\n",
                "   \n",
                "# comment\n",
                "invalid syntax\n",
                "impor\n",
                "export foo\n",
                "Import foo\n",
                "+++---\n",
                "123 456\n",
            ];
            for input_str in cases {
                let input = InputSpan::new_extra(input_str, Config::default());
                let error = match parse(input) {
                    Err(nom::Err::Error(e) | nom::Err::Failure(e)) => e,
                    Ok(_) => panic!("Expected error for {input_str:?}"),
                    Err(e) => panic!("Unexpected nom result for {input_str:?}: {e:?}"),
                };
                assert_eq!(
                    error.reason,
                    ParserErrorReason::Expect(ExpectKind::Decl),
                    "reason for {input_str:?}"
                );
            }
        }

        #[test]
        fn import_incomplete_errors() {
            use ImportKind::*;
            let cases: &[(&str, usize, ImportKind, usize, usize)] = &[
                ("import\n", 6, MissingPath, 0, 6),
                ("import 123\n", 7, MissingPath, 0, 6),
                ("import foo@bar\n", 10, MissingEndOfLine, 7, 10),
            ];
            for &(input_str, offset, ref import_kind, cs, ce) in cases {
                assert_failure(
                    input_str,
                    offset,
                    IncompleteKind::Decl(DeclKind::Import(*import_kind)),
                    cs,
                    ce,
                );
            }
        }

        #[test]
        fn submodel_incomplete_errors() {
            // (input, error_offset, expected_kind, cause_start, cause_end)
            let cases: &[(&str, usize, IncompleteKind, usize, usize)] = &[
                (
                    "submodel foo as\n",
                    15,
                    IncompleteKind::Decl(DeclKind::AsMissingAlias),
                    13,
                    15,
                ),
                (
                    "submodel 123\n",
                    9,
                    IncompleteKind::Decl(DeclKind::Submodel(SubmodelKind::MissingModelInfo {
                        after: SubmodelKeyword::Submodel,
                    })),
                    0,
                    8,
                ),
                (
                    "submodel foo as 123\n",
                    16,
                    IncompleteKind::Decl(DeclKind::AsMissingAlias),
                    13,
                    15,
                ),
            ];
            for &(input_str, offset, ref expected_kind, cs, ce) in cases {
                assert_failure(input_str, offset, *expected_kind, cs, ce);
            }
        }

        #[test]
        fn model_info_expect_identifier_errors() {
            let cases: &[&str] = &["", "123.bar"];
            for input_str in cases {
                let input = InputSpan::new_extra(input_str, Config::default());
                let Err(nom::Err::Error(error)) = model_info(input) else {
                    panic!("Expected Error for {input_str:?}");
                };
                assert_eq!(error.error_offset, 0, "offset for {input_str:?}");
                assert_eq!(
                    error.reason,
                    ParserErrorReason::TokenError(TokenErrorKind::Expect(
                        TokenExpectKind::Identifier
                    )),
                    "reason for {input_str:?}"
                );
            }
        }

        #[test]
        fn model_info_missing_subcomponent() {
            let cases: &[(&str, usize, usize, usize)] = &[
                ("foo.", 4, 3, 4),
                ("foo.123", 4, 3, 4),
                ("foo.bar.", 8, 7, 8),
            ];
            for &(input_str, offset, cs, ce) in cases {
                let input = InputSpan::new_extra(input_str, Config::default());
                let Err(nom::Err::Failure(error)) = model_info(input) else {
                    panic!("Expected Failure for {input_str:?}");
                };
                assert_eq!(error.error_offset, offset, "offset for {input_str:?}");
                let ParserErrorReason::Incomplete { kind, cause } = error.reason else {
                    panic!(
                        "Expected Incomplete for {input_str:?}, got {:?}",
                        error.reason
                    );
                };
                assert_eq!(
                    kind,
                    IncompleteKind::Decl(DeclKind::ModelMissingSubcomponent),
                    "kind for {input_str:?}"
                );
                assert_eq!(cause.start().offset, cs, "cause_start for {input_str:?}");
                assert_eq!(cause.end().offset, ce, "cause_end for {input_str:?}");
            }
        }

        #[test]
        fn parse_complete_with_remaining_input() {
            let input = InputSpan::new_extra("import foo\nrest", Config::default());
            let result = parse_complete(input);

            let Err(nom::Err::Error(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 11);
            assert_eq!(error.reason, ParserErrorReason::UnexpectedToken);
        }

        #[test]
        fn unclosed_bracket_errors() {
            // (input, error_offset, cause_start, cause_end)
            let cases: &[(&str, usize, usize, usize)] = &[
                ("submodel foo [\n", 15, 13, 14),
                ("submodel foo [bar\n", 18, 13, 14),
                ("submodel foo [bar, baz\n", 23, 13, 14),
                ("submodel foo [bar, baz,\n", 24, 13, 14),
                ("submodel foo [bar.qux, baz.quux\n", 32, 13, 14),
                ("submodel foo [bar as baz, qux as quux\n", 38, 13, 14),
                ("submodel foo as bar [qux, baz\n", 30, 20, 21),
                ("submodel foo [\nbar,\nbaz\n", 24, 13, 14),
                (
                    "submodel utils/math [trigonometry as trig, sin, cos as cosine\n",
                    62,
                    20,
                    21,
                ),
            ];
            for &(input_str, offset, cs, ce) in cases {
                assert_failure(input_str, offset, IncompleteKind::UnclosedBracket, cs, ce);
            }
        }

        #[test]
        fn submodel_parses_in_design_context() {
            // In design context (`allow_design_shorthand` set), `submodel model as alias`
            // parses as Submodel just like in regular context.
            let config = Config {
                allow_design_shorthand: true,
                ..Config::default()
            };
            let input = InputSpan::new_extra("submodel foo as bar\n", config);
            let result = parse(input);
            let Ok((rest, decl)) = result else {
                panic!("Expected Ok, got {result:?}");
            };
            assert!(rest.fragment().is_empty() || rest.fragment().chars().all(char::is_whitespace));
            assert!(
                matches!(&*decl, Decl::Submodel(_)),
                "Expected Submodel, got {decl:?}",
            );
        }

        #[test]
        fn submodel_parses_in_regular_context() {
            let input = InputSpan::new_extra("submodel foo as bar\n", Config::default());
            let result = parse(input);
            let Ok((rest, decl)) = result else {
                panic!("Expected Ok, got {result:?}");
            };
            assert!(rest.fragment().is_empty() || rest.fragment().chars().all(char::is_whitespace));
            assert!(
                matches!(&*decl, Decl::Submodel(_)),
                "Expected Submodel, got {decl:?}",
            );
        }

        #[test]
        fn submodel_with_extracted_submodels_single() {
            let input = InputSpan::new_extra("submodel foo as bar [baz]\n", Config::default());
            let result = parse(input);
            let Ok((rest, decl)) = result else {
                panic!("Expected Ok, got {result:?}");
            };
            assert!(rest.fragment().is_empty() || rest.fragment().chars().all(char::is_whitespace));
            let Decl::Submodel(um) = &*decl else {
                panic!("Expected Submodel, got {:?}", &*decl);
            };
            assert_eq!(um.model_info().top_component().as_str(), "foo");
            assert_eq!(um.model_info().get_alias().as_str(), "bar");
            let submodels = um.imported_submodels().expect("Expected submodel list");
            assert_eq!(submodels.len(), 1);
            assert_eq!(submodels[0].get_model_name().as_str(), "baz");
        }

        #[test]
        fn submodel_with_extracted_submodels_list() {
            let input = InputSpan::new_extra("submodel foo as bar [baz, qux]\n", Config::default());
            let result = parse(input);
            let Ok((rest, decl)) = result else {
                panic!("Expected Ok, got {result:?}");
            };
            assert!(rest.fragment().is_empty() || rest.fragment().chars().all(char::is_whitespace));
            let Decl::Submodel(um) = &*decl else {
                panic!("Expected Submodel, got {:?}", &*decl);
            };
            assert_eq!(um.model_info().top_component().as_str(), "foo");
            assert_eq!(um.model_info().get_alias().as_str(), "bar");
            let submodels = um.imported_submodels().expect("Expected submodel list");
            assert_eq!(submodels.len(), 2);
            assert_eq!(submodels[0].get_model_name().as_str(), "baz");
            assert_eq!(submodels[1].get_model_name().as_str(), "qux");
        }

        #[test]
        fn submodel_without_extracted_submodels() {
            let input = InputSpan::new_extra("submodel foo as bar\n", Config::default());
            let result = parse(input);
            let Ok((_, decl)) = result else {
                panic!("Expected Ok, got {result:?}");
            };
            let Decl::Submodel(um) = &*decl else {
                panic!("Expected Submodel, got {:?}", &*decl);
            };
            assert!(um.imported_submodels().is_none());
        }

        #[test]
        fn apply_missing_target() {
            let input = InputSpan::new_extra("apply uhf\n", Config::default());
            let result = parse(input);
            let Err(nom::Err::Failure(e)) = result else {
                panic!("Expected failure for missing target, got {result:?}");
            };
            let ParserErrorReason::Incomplete { kind, .. } = e.reason else {
                panic!("Unexpected reason {:?}", e.reason);
            };
            assert_eq!(
                kind,
                IncompleteKind::Decl(DeclKind::ApplyMissingTarget),
                "expected ApplyMissingTarget"
            );
        }
    }
}
