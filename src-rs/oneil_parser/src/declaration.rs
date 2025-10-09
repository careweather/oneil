//! Parser for declarations in an Oneil program.

use nom::{
    Parser as _,
    branch::alt,
    combinator::{all_consuming, opt},
    multi::many0,
};

use oneil_ast::{
    Decl, DeclNode, Directory, DirectoryNode, IdentifierNode, Import, ModelInfo, ModelInfoNode,
    ModelKind, Node, SubmodelList, SubmodelListNode, UseModel,
};
use oneil_shared::span::Span;

use crate::{
    error::{ErrorHandlingParser, ParserError},
    parameter::parse as parse_parameter,
    test::parse as parse_test,
    token::{
        keyword::{as_, import, ref_, use_, with},
        naming::identifier,
        structure::end_of_line,
        symbol::{bracket_left, bracket_right, comma, dot, dot_dot, slash},
    },
    util::{InputSpan, Result},
};

/// Parses a declaration
///
/// This function **may not consume the complete input**.
pub fn parse(input: InputSpan<'_>) -> Result<'_, DeclNode, ParserError> {
    decl.parse(input)
}

/// Parses a declaration
///
/// This function **fails if the complete input is not consumed**.
pub fn parse_complete(input: InputSpan<'_>) -> Result<'_, DeclNode, ParserError> {
    all_consuming(decl).parse(input)
}

/// Parses any type of declaration by trying each declaration parser in sequence.
///
/// This function attempts to parse the input as each type of declaration:
/// 1. Import declaration (`import path`)
/// 2. From declaration (`from path use model as alias`)
/// 3. Use declaration (`use path as alias`)
/// 4. Parameter declaration (parameter definitions)
/// 5. Test declaration (`test: condition`)
///
/// The first parser that succeeds determines the declaration type.
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns a declaration node of the appropriate type, or an error if no
/// declaration type matches.
fn decl(input: InputSpan<'_>) -> Result<'_, DeclNode, ParserError> {
    alt((import_decl, use_decl, parameter_decl, test_decl))
        .convert_error_to(ParserError::expect_decl)
        .parse(input)
}

/// Parses an import declaration
fn import_decl(input: InputSpan<'_>) -> Result<'_, DeclNode, ParserError> {
    let (rest, import_token) = import.convert_errors().parse(input)?;

    // TODO: allow a path here (ex. `import foo.bar`)
    let (rest, import_path_token) = identifier
        .or_fail_with(ParserError::import_missing_path(import_token.lexeme_span))
        .parse(rest)?;

    let (rest, end_of_line_token) = end_of_line
        .or_fail_with(ParserError::import_missing_end_of_line(
            import_path_token.lexeme_span,
        ))
        .parse(rest)?;

    let node_span =
        Span::from_start_and_end(&import_token.lexeme_span, &end_of_line_token.lexeme_span);
    let whitespace_span = end_of_line_token.whitespace_span;

    let import_path_str = Node::<String>::from(import_path_token);

    let import_node = Node::new(Import::new(import_path_str), node_span, whitespace_span);

    let decl_node = Node::new(Decl::Import(import_node), node_span, whitespace_span);

    Ok((rest, decl_node))
}

/// Parses a use declaration
fn use_decl(input: InputSpan<'_>) -> Result<'_, DeclNode, ParserError> {
    let ref_keyword = |input| {
        let (rest, ref_token) = ref_.convert_errors().parse(input)?;
        Ok((rest, (ModelKind::Reference, ref_token)))
    };

    let use_keyword = |input| {
        let (rest, use_token) = use_.convert_errors().parse(input)?;
        Ok((rest, (ModelKind::Submodel, use_token)))
    };

    // either parse the ref keyword or the use keyword
    let (rest, (is_ref_only, keyword_token)) = alt((ref_keyword, use_keyword)).parse(input)?;

    let (rest, directory_path) = opt_directory_path.parse(rest)?;

    let (rest, model_info) = model_info
        .or_fail_with(ParserError::use_missing_model_info(
            keyword_token.lexeme_span,
        ))
        .parse(rest)?;

    let (rest, submodel_list) = opt(|input| {
        let (rest, _with_token) = with.convert_errors().parse(input)?;
        submodel_list(rest)
    })
    .parse(rest)?;

    let final_span = submodel_list
        .as_ref()
        .map_or_else(|| model_info.span(), Node::span);

    let (rest, end_of_line_token) = end_of_line
        .or_fail_with(ParserError::use_missing_end_of_line(final_span))
        .parse(rest)?;

    let use_model_node = Node::new(
        UseModel::new(directory_path, model_info, submodel_list, is_ref_only),
        final_span,
        end_of_line_token.whitespace_span,
    );

    let decl_node = Node::new(
        Decl::UseModel(use_model_node),
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

pub fn model_info(input: InputSpan<'_>) -> Result<'_, ModelInfoNode, ParserError> {
    let (rest, top_component_token) = identifier.convert_errors().parse(input)?;
    let top_component_node = IdentifierNode::from(top_component_token);

    let (rest, subcomponents) = opt_subcomponents.parse(rest)?;
    let (rest, alias) = opt(as_alias).parse(rest)?;

    let (final_span, whitespace_span) = match (subcomponents.last(), &alias) {
        (_, Some(alias)) => (alias.span(), alias.whitespace_span()),
        (Some(subcomponent), None) => (subcomponent.span(), subcomponent.whitespace_span()),
        (None, None) => (
            top_component_node.span(),
            top_component_node.whitespace_span(),
        ),
    };

    let model_info_span = Span::from_start_and_end(&top_component_node.span(), &final_span);
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

/// Parses a list of submodels in a use declaration
fn submodel_list(input: InputSpan<'_>) -> Result<'_, SubmodelListNode, ParserError> {
    let single_submodel = |input| {
        let (rest, submodel) = model_info.parse(input)?;
        let submodel_span = submodel.span();
        let submodel_whitespace_span = submodel.whitespace_span();

        let submodel_list = SubmodelList::new(vec![submodel]);
        let submodel_list_node = Node::new(submodel_list, submodel_span, submodel_whitespace_span);

        Ok((rest, submodel_list_node))
    };

    let multiple_submodels = |input| {
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
            let (rest, _optional_end_of_line_token) =
                opt(end_of_line).convert_errors().parse(rest)?;

            let mut submodels = rest_submodels;
            submodels.insert(0, first_submodel);
            Ok((rest, submodels))
        })
        .parse(rest)?;

        let (rest, bracket_right_token) = bracket_right
            .or_fail_with(ParserError::unclosed_bracket(
                bracket_left_token.lexeme_span,
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
    };

    alt((single_submodel, multiple_submodels)).parse(input)
}

/// Parses a parameter declaration by delegating to the parameter parser.
fn parameter_decl(input: InputSpan<'_>) -> Result<'_, DeclNode, ParserError> {
    let (rest, parameter) = parse_parameter.parse(input)?;

    let parameter_span = parameter.span();
    let parameter_whitespace_span = parameter.whitespace_span();
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

    let span = test.span();
    let whitespace_span = test.whitespace_span();
    let decl_node = Node::new(Decl::Test(test), span, whitespace_span);

    Ok((rest, decl_node))
}

#[cfg(test)]
#[expect(
    clippy::similar_names,
    reason = "tests should make it clear what variable is being tested"
)]
mod tests {
    use super::*;
    use crate::Config;
    use crate::util::test::assert_node_contains;

    mod success {
        use super::*;

        #[test]
        fn import_decl() {
            let input = InputSpan::new_extra("import foo\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::Import(ref import_node) = *decl else {
                panic!("Expected import declaration");
            };

            let import_path = import_node.path();
            assert_node_contains!(import_path, "foo".to_string(), start_offset: 7, end_offset: 10);

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn ref_decl() {
            let input = InputSpan::new_extra("ref foo\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::UseModel(ref use_model_node) = *decl else {
                panic!("Expected use declaration");
            };

            let use_model_info = use_model_node.model_info();
            assert_eq!(use_model_info.top_component().as_str(), "foo");
            assert_eq!(use_model_info.subcomponents().len(), 0);
            assert_eq!(use_model_info.get_alias().as_str(), "foo");
            assert_eq!(use_model_node.model_kind(), ModelKind::Reference);

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn use_decl() {
            let input = InputSpan::new_extra("use foo.bar as baz\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::UseModel(ref use_model_node) = *decl else {
                panic!("Expected use declaration");
            };

            let use_model_info = use_model_node.model_info();
            assert_eq!(use_model_info.top_component().as_str(), "foo");
            assert_eq!(use_model_info.subcomponents()[0].as_str(), "bar");
            assert_eq!(use_model_info.get_alias().as_str(), "baz");
            assert_eq!(use_model_node.model_kind(), ModelKind::Submodel);

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn use_decl_without_alias() {
            let input = InputSpan::new_extra("use foo.bar\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::UseModel(ref use_model_node) = *decl else {
                panic!("Expected use declaration");
            };

            let use_model_info = use_model_node.model_info();
            assert_eq!(use_model_info.top_component().as_str(), "foo");
            assert_eq!(use_model_info.subcomponents().len(), 1);
            assert_eq!(use_model_info.subcomponents()[0].as_str(), "bar");
            assert_eq!(use_model_info.get_alias().as_str(), "bar");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn use_decl_simple_without_alias() {
            let input = InputSpan::new_extra("use foo\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::UseModel(ref use_model_node) = *decl else {
                panic!("Expected use declaration");
            };

            let use_model_info = use_model_node.model_info();
            assert_eq!(use_model_info.top_component().as_str(), "foo");
            assert_eq!(use_model_info.subcomponents().len(), 0);
            assert_eq!(use_model_info.get_alias().as_str(), "foo");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn parse_complete_import_success() {
            let input = InputSpan::new_extra("import foo\n", Config::default());
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");

            let Decl::Import(ref import_node) = *decl else {
                panic!("Expected import declaration");
            };

            let import_path_node = import_node.path();
            assert_node_contains!(import_path_node, "foo", start_offset: 7, end_offset: 10);

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn parse_complete_use_success() {
            let input = InputSpan::new_extra("use foo.bar as baz\n", Config::default());
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");

            let Decl::UseModel(ref use_model_node) = *decl else {
                panic!("Expected use declaration");
            };

            let use_model_info = use_model_node.model_info();
            assert_eq!(use_model_info.top_component().as_str(), "foo");
            assert_eq!(use_model_info.subcomponents()[0].as_str(), "bar");
            assert_eq!(use_model_info.get_alias().as_str(), "baz");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn use_with_single_directory() {
            let input = InputSpan::new_extra("use utils/math as calculator\n", Config::default());
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");

            let Decl::UseModel(ref use_model_node) = *decl else {
                panic!("Expected use declaration");
            };

            let use_model_info = use_model_node.model_info();
            assert_eq!(use_model_info.top_component().as_str(), "math");
            assert_eq!(use_model_info.subcomponents().len(), 0);
            assert_eq!(use_model_info.get_alias().as_str(), "calculator");

            // Check directory path
            assert_eq!(use_model_node.directory_path().len(), 1);
            assert_eq!(use_model_node.directory_path()[0].as_str(), "utils");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn use_with_single_directory_without_alias() {
            let input = InputSpan::new_extra("use utils/math\n", Config::default());
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");

            let Decl::UseModel(ref use_model_node) = *decl else {
                panic!("Expected use declaration");
            };

            let use_model_info = use_model_node.model_info();
            assert_eq!(use_model_info.top_component().as_str(), "math");
            assert_eq!(use_model_info.subcomponents().len(), 0);
            assert_eq!(use_model_info.get_alias().as_str(), "math");

            // Check directory path
            assert_eq!(use_model_node.directory_path().len(), 1);
            assert_eq!(use_model_node.directory_path()[0].as_str(), "utils");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn use_with_multiple_directories() {
            let input = InputSpan::new_extra(
                "use models/physics/mechanics as dynamics\n",
                Config::default(),
            );
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");

            let Decl::UseModel(ref use_model_node) = *decl else {
                panic!("Expected use declaration");
            };

            let use_model_info = use_model_node.model_info();
            assert_eq!(use_model_info.top_component().as_str(), "mechanics");
            assert_eq!(use_model_info.subcomponents().len(), 0);
            assert_eq!(use_model_info.get_alias().as_str(), "dynamics");

            // Check directory path
            assert_eq!(use_model_node.directory_path().len(), 2);
            assert_eq!(use_model_node.directory_path()[0].as_str(), "models");
            assert_eq!(use_model_node.directory_path()[1].as_str(), "physics");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn use_with_directory_and_subcomponents() {
            let input =
                InputSpan::new_extra("use utils/math.trigonometry as trig\n", Config::default());
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");

            let Decl::UseModel(ref use_model_node) = *decl else {
                panic!("Expected use declaration");
            };

            let use_model_info = use_model_node.model_info();
            assert_eq!(use_model_info.top_component().as_str(), "math");
            assert_eq!(use_model_info.subcomponents().len(), 1);
            assert_eq!(use_model_info.subcomponents()[0].as_str(), "trigonometry");
            assert_eq!(use_model_info.get_alias().as_str(), "trig");

            // Check directory path
            assert_eq!(use_model_node.directory_path().len(), 1);
            assert_eq!(use_model_node.directory_path()[0].as_str(), "utils");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn use_with_current_directory() {
            let input = InputSpan::new_extra("use ./local_model as local\n", Config::default());
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");

            let Decl::UseModel(ref use_model_node) = *decl else {
                panic!("Expected use declaration");
            };

            let use_model_info = use_model_node.model_info();
            assert_eq!(use_model_info.top_component().as_str(), "local_model");
            assert_eq!(use_model_info.subcomponents().len(), 0);
            assert_eq!(use_model_info.get_alias().as_str(), "local");

            // Check directory path
            assert_eq!(use_model_node.directory_path().len(), 1);
            assert_eq!(use_model_node.directory_path()[0].as_str(), ".");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn use_with_parent_directory() {
            let input = InputSpan::new_extra("use ../parent_model as parent\n", Config::default());
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");

            let Decl::UseModel(ref use_model_node) = *decl else {
                panic!("Expected use declaration");
            };

            let use_model_info = use_model_node.model_info();
            assert_eq!(use_model_info.top_component().as_str(), "parent_model");
            assert_eq!(use_model_info.subcomponents().len(), 0);
            assert_eq!(use_model_info.get_alias().as_str(), "parent");

            // Check directory path
            assert_eq!(use_model_node.directory_path().len(), 1);
            assert_eq!(use_model_node.directory_path()[0].as_str(), "..");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn use_with_mixed_directory_types() {
            let input = InputSpan::new_extra(
                "use ../shared/./utils/math as shared_math\n",
                Config::default(),
            );
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");

            let Decl::UseModel(ref use_model_node) = *decl else {
                panic!("Expected use declaration");
            };

            let use_model_info = use_model_node.model_info();
            assert_eq!(use_model_info.top_component().as_str(), "math");
            assert_eq!(use_model_info.subcomponents().len(), 0);
            assert_eq!(use_model_info.get_alias().as_str(), "shared_math");

            // Check directory path
            assert_eq!(use_model_node.directory_path().len(), 4);
            assert_eq!(use_model_node.directory_path()[0].as_str(), "..");
            assert_eq!(use_model_node.directory_path()[1].as_str(), "shared");
            assert_eq!(use_model_node.directory_path()[2].as_str(), ".");
            assert_eq!(use_model_node.directory_path()[3].as_str(), "utils");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn use_with_complex_path_and_subcomponents() {
            let input = InputSpan::new_extra(
                "use models/physics/mechanics.rotational.dynamics as rotation\n",
                Config::default(),
            );
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");

            let Decl::UseModel(ref use_model_node) = *decl else {
                panic!("Expected use declaration");
            };

            let use_model_info = use_model_node.model_info();
            assert_eq!(use_model_info.top_component().as_str(), "mechanics");
            assert_eq!(use_model_info.subcomponents().len(), 2);
            assert_eq!(use_model_info.subcomponents()[0].as_str(), "rotational");
            assert_eq!(use_model_info.subcomponents()[1].as_str(), "dynamics");
            assert_eq!(use_model_info.get_alias().as_str(), "rotation");

            // Check directory path
            assert_eq!(use_model_node.directory_path().len(), 2);
            assert_eq!(use_model_node.directory_path()[0].as_str(), "models");
            assert_eq!(use_model_node.directory_path()[1].as_str(), "physics");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn directory_name_parsing() {
            // Test parent directory
            let input = InputSpan::new_extra("..", Config::default());
            let (rest, dir) = directory_name(input).expect("should parse parent directory");
            assert_eq!(dir.as_str(), "..");
            assert_eq!(rest.fragment(), &"");

            // Test current directory
            let input = InputSpan::new_extra(".", Config::default());
            let (rest, dir) = directory_name(input).expect("should parse current directory");
            assert_eq!(dir.as_str(), ".");
            assert_eq!(rest.fragment(), &"");

            // Test regular directory name
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
        fn use_decl_with_single_submodel() {
            let input = InputSpan::new_extra("use foo with bar\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::UseModel(ref use_model_node) = *decl else {
                panic!("Expected use declaration");
            };

            let use_model_info = use_model_node.model_info();
            assert_eq!(use_model_info.top_component().as_str(), "foo");
            assert_eq!(use_model_info.subcomponents().len(), 0);
            assert_eq!(use_model_info.get_alias().as_str(), "foo");

            // Check submodels
            let submodels = use_model_node.submodels().expect("should have submodels");
            assert_eq!(submodels.len(), 1);

            let submodel = &submodels[0];
            assert_eq!(submodel.top_component().as_str(), "bar");
            assert_eq!(submodel.subcomponents().len(), 0);
            assert_eq!(submodel.get_alias().as_str(), "bar");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn use_decl_with_single_submodel_with_alias() {
            let input = InputSpan::new_extra("use foo with bar as baz\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::UseModel(ref use_model_node) = *decl else {
                panic!("Expected use declaration");
            };

            let use_model_info = use_model_node.model_info();
            assert_eq!(use_model_info.top_component().as_str(), "foo");
            assert_eq!(use_model_info.subcomponents().len(), 0);
            assert_eq!(use_model_info.get_alias().as_str(), "foo");

            // Check submodels
            let submodels = use_model_node.submodels().expect("should have submodels");
            assert_eq!(submodels.len(), 1);

            let submodel = &submodels[0];
            assert_eq!(submodel.top_component().as_str(), "bar");
            assert_eq!(submodel.subcomponents().len(), 0);
            assert_eq!(submodel.get_alias().as_str(), "baz");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn use_decl_with_single_submodel_with_subcomponents() {
            let input = InputSpan::new_extra("use foo with bar.qux\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::UseModel(ref use_model_node) = *decl else {
                panic!("Expected use declaration");
            };

            let use_model_info = use_model_node.model_info();
            assert_eq!(use_model_info.top_component().as_str(), "foo");
            assert_eq!(use_model_info.subcomponents().len(), 0);
            assert_eq!(use_model_info.get_alias().as_str(), "foo");

            // Check submodels
            let submodels = use_model_node.submodels().expect("should have submodels");
            assert_eq!(submodels.len(), 1);

            let submodel = &submodels[0];
            assert_eq!(submodel.top_component().as_str(), "bar");
            assert_eq!(submodel.subcomponents().len(), 1);
            assert_eq!(submodel.subcomponents()[0].as_str(), "qux");
            assert_eq!(submodel.get_alias().as_str(), "qux");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn use_decl_with_multiple_submodels() {
            let input = InputSpan::new_extra("use foo with [bar, qux]\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::UseModel(ref use_model_node) = *decl else {
                panic!("Expected use declaration");
            };

            let use_model_info = use_model_node.model_info();
            assert_eq!(use_model_info.top_component().as_str(), "foo");
            assert_eq!(use_model_info.subcomponents().len(), 0);
            assert_eq!(use_model_info.get_alias().as_str(), "foo");

            // Check submodels
            let submodels = use_model_node.submodels().expect("should have submodels");
            assert_eq!(submodels.len(), 2);

            let submodel1 = &submodels[0];
            assert_eq!(submodel1.top_component().as_str(), "bar");
            assert_eq!(submodel1.subcomponents().len(), 0);
            assert_eq!(submodel1.get_alias().as_str(), "bar");

            let submodel2 = &submodels[1];
            assert_eq!(submodel2.top_component().as_str(), "qux");
            assert_eq!(submodel2.subcomponents().len(), 0);
            assert_eq!(submodel2.get_alias().as_str(), "qux");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn use_decl_with_multiple_submodels_with_aliases() {
            let input = InputSpan::new_extra(
                "use foo with [bar as baz, qux as quux]\n",
                Config::default(),
            );
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::UseModel(ref use_model_node) = *decl else {
                panic!("Expected use declaration");
            };

            let use_model_info = use_model_node.model_info();
            assert_eq!(use_model_info.top_component().as_str(), "foo");
            assert_eq!(use_model_info.subcomponents().len(), 0);
            assert_eq!(use_model_info.get_alias().as_str(), "foo");

            // Check submodels
            let submodels = use_model_node.submodels().expect("should have submodels");
            assert_eq!(submodels.len(), 2);

            let submodel1 = &submodels[0];
            assert_eq!(submodel1.top_component().as_str(), "bar");
            assert_eq!(submodel1.subcomponents().len(), 0);
            assert_eq!(submodel1.get_alias().as_str(), "baz");

            let submodel2 = &submodels[1];
            assert_eq!(submodel2.top_component().as_str(), "qux");
            assert_eq!(submodel2.subcomponents().len(), 0);
            assert_eq!(submodel2.get_alias().as_str(), "quux");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn use_decl_with_multiple_submodels_with_subcomponents() {
            let input =
                InputSpan::new_extra("use foo with [bar.qux, baz.quux.quuz]\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::UseModel(ref use_model_node) = *decl else {
                panic!("Expected use declaration");
            };

            let use_model_info = use_model_node.model_info();
            assert_eq!(use_model_info.top_component().as_str(), "foo");
            assert_eq!(use_model_info.subcomponents().len(), 0);
            assert_eq!(use_model_info.get_alias().as_str(), "foo");

            // Check submodels
            let submodels = use_model_node.submodels().expect("should have submodels");
            assert_eq!(submodels.len(), 2);

            let submodel1 = &submodels[0];
            assert_eq!(submodel1.top_component().as_str(), "bar");
            assert_eq!(submodel1.subcomponents().len(), 1);
            assert_eq!(submodel1.subcomponents()[0].as_str(), "qux");
            assert_eq!(submodel1.get_alias().as_str(), "qux");

            let submodel2 = &submodels[1];
            assert_eq!(submodel2.top_component().as_str(), "baz");
            assert_eq!(submodel2.subcomponents().len(), 2);
            assert_eq!(submodel2.subcomponents()[0].as_str(), "quux");
            assert_eq!(submodel2.subcomponents()[1].as_str(), "quuz");
            assert_eq!(submodel2.get_alias().as_str(), "quuz");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn use_decl_with_multiple_submodels_with_trailing_comma() {
            let input = InputSpan::new_extra("use foo with [bar, qux,]\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::UseModel(ref use_model_node) = *decl else {
                panic!("Expected use declaration");
            };

            let use_model_info = use_model_node.model_info();
            assert_eq!(use_model_info.top_component().as_str(), "foo");
            assert_eq!(use_model_info.subcomponents().len(), 0);
            assert_eq!(use_model_info.get_alias().as_str(), "foo");

            // Check submodels
            let submodels = use_model_node.submodels().expect("should have submodels");
            assert_eq!(submodels.len(), 2);

            let submodel1 = &submodels[0];
            assert_eq!(submodel1.top_component().as_str(), "bar");
            assert_eq!(submodel1.subcomponents().len(), 0);
            assert_eq!(submodel1.get_alias().as_str(), "bar");

            let submodel2 = &submodels[1];
            assert_eq!(submodel2.top_component().as_str(), "qux");
            assert_eq!(submodel2.subcomponents().len(), 0);
            assert_eq!(submodel2.get_alias().as_str(), "qux");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn use_decl_with_empty_submodel_list() {
            let input = InputSpan::new_extra("use foo with []\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::UseModel(ref use_model_node) = *decl else {
                panic!("Expected use declaration");
            };

            let use_model_info = use_model_node.model_info();
            assert_eq!(use_model_info.top_component().as_str(), "foo");
            assert_eq!(use_model_info.subcomponents().len(), 0);
            assert_eq!(use_model_info.get_alias().as_str(), "foo");

            // Check submodels - should be empty
            let submodels = use_model_node.submodels().expect("should have submodels");
            assert_eq!(submodels.len(), 0);

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn use_decl_with_model_alias_and_submodels() {
            let input = InputSpan::new_extra("use foo as bar with [qux, baz]\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::UseModel(ref use_model_node) = *decl else {
                panic!("Expected use declaration");
            };

            let use_model_info = use_model_node.model_info();
            assert_eq!(use_model_info.top_component().as_str(), "foo");
            assert_eq!(use_model_info.subcomponents().len(), 0);
            assert_eq!(use_model_info.get_alias().as_str(), "bar");

            // Check submodels
            let submodels = use_model_node.submodels().expect("should have submodels");
            assert_eq!(submodels.len(), 2);

            let submodel1 = &submodels[0];
            assert_eq!(submodel1.top_component().as_str(), "qux");
            assert_eq!(submodel1.subcomponents().len(), 0);
            assert_eq!(submodel1.get_alias().as_str(), "qux");

            let submodel2 = &submodels[1];
            assert_eq!(submodel2.top_component().as_str(), "baz");
            assert_eq!(submodel2.subcomponents().len(), 0);
            assert_eq!(submodel2.get_alias().as_str(), "baz");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn use_decl_with_complex_path_and_submodels() {
            let input = InputSpan::new_extra(
                "use utils/math.trigonometry as trig with [sin, cos as cosine]\n",
                Config::default(),
            );
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::UseModel(ref use_model_node) = *decl else {
                panic!("Expected use declaration");
            };

            let use_model_info = use_model_node.model_info();
            assert_eq!(use_model_info.top_component().as_str(), "math");
            assert_eq!(use_model_info.subcomponents().len(), 1);
            assert_eq!(use_model_info.subcomponents()[0].as_str(), "trigonometry");
            assert_eq!(use_model_info.get_alias().as_str(), "trig");

            // Check directory path
            assert_eq!(use_model_node.directory_path().len(), 1);
            assert_eq!(use_model_node.directory_path()[0].as_str(), "utils");

            // Check submodels
            let submodels = use_model_node.submodels().expect("should have submodels");
            assert_eq!(submodels.len(), 2);

            let submodel1 = &submodels[0];
            assert_eq!(submodel1.top_component().as_str(), "sin");
            assert_eq!(submodel1.subcomponents().len(), 0);
            assert_eq!(submodel1.get_alias().as_str(), "sin");

            let submodel2 = &submodels[1];
            assert_eq!(submodel2.top_component().as_str(), "cos");
            assert_eq!(submodel2.subcomponents().len(), 0);
            assert_eq!(submodel2.get_alias().as_str(), "cosine");

            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn use_decl_with_submodels_and_newlines() {
            let input = InputSpan::new_extra("use foo with [\nbar,\nqux\n]\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");

            let Decl::UseModel(ref use_model_node) = *decl else {
                panic!("Expected use declaration");
            };

            let use_model_info = use_model_node.model_info();
            assert_eq!(use_model_info.top_component().as_str(), "foo");
            assert_eq!(use_model_info.subcomponents().len(), 0);
            assert_eq!(use_model_info.get_alias().as_str(), "foo");

            // Check submodels
            let submodels = use_model_node.submodels().expect("should have submodels");
            assert_eq!(submodels.len(), 2);

            let submodel1 = &submodels[0];
            assert_eq!(submodel1.top_component().as_str(), "bar");
            assert_eq!(submodel1.subcomponents().len(), 0);
            assert_eq!(submodel1.get_alias().as_str(), "bar");

            let submodel2 = &submodels[1];
            assert_eq!(submodel2.top_component().as_str(), "qux");
            assert_eq!(submodel2.subcomponents().len(), 0);
            assert_eq!(submodel2.get_alias().as_str(), "qux");

            assert_eq!(rest.fragment(), &"");
        }
    }

    mod error {
        use super::*;

        mod import_error_tests {
            use crate::error::reason::{
                DeclKind, ExpectKind, ImportKind, IncompleteKind, ParserErrorReason,
            };

            use super::*;

            #[test]
            fn empty_input() {
                let input = InputSpan::new_extra("", Config::default());
                let result = parse(input);

                let Err(nom::Err::Error(error)) = result else {
                    panic!("Expected error for empty input");
                };

                assert_eq!(error.error_offset, 0);
                assert!(matches!(
                    error.reason,
                    ParserErrorReason::Expect(ExpectKind::Decl)
                ));
            }

            #[test]
            fn missing_import_keyword() {
                let input = InputSpan::new_extra("foo\n", Config::default());
                let result = parse(input);

                let Err(nom::Err::Error(error)) = result else {
                    panic!("Expected error for missing import keyword");
                };

                assert_eq!(error.error_offset, 0);
                assert!(matches!(
                    error.reason,
                    ParserErrorReason::Expect(ExpectKind::Decl)
                ));
            }

            #[test]
            fn missing_path() {
                let input = InputSpan::new_extra("import\n", Config::default());
                let result = parse(input);

                let Err(nom::Err::Failure(error)) = result else {
                    panic!("Unexpected result {result:?}");
                };

                assert_eq!(error.error_offset, 6);

                let ParserErrorReason::Incomplete {
                    kind: IncompleteKind::Decl(DeclKind::Import(ImportKind::MissingPath)),
                    cause,
                } = error.reason
                else {
                    panic!("Unexpected reason {:?}", error.reason);
                };

                assert_eq!(cause.start().offset, 0);
                assert_eq!(cause.end().offset, 6);
            }

            #[test]
            fn invalid_path_identifier() {
                let input = InputSpan::new_extra("import 123\n", Config::default());
                let result = parse(input);

                let Err(nom::Err::Failure(error)) = result else {
                    panic!("Unexpected result {result:?}");
                };

                assert_eq!(error.error_offset, 7);

                let ParserErrorReason::Incomplete {
                    kind: IncompleteKind::Decl(DeclKind::Import(ImportKind::MissingPath)),
                    cause,
                } = error.reason
                else {
                    panic!("Unexpected reason {:?}", error.reason);
                };

                assert_eq!(cause.start().offset, 0);
                assert_eq!(cause.end().offset, 6);
            }

            #[test]
            fn path_with_missing_end_of_line() {
                let input = InputSpan::new_extra("import foo@bar\n", Config::default());
                let result = parse(input);

                let Err(nom::Err::Failure(error)) = result else {
                    panic!("Unexpected result {result:?}");
                };

                assert_eq!(error.error_offset, 10);

                let ParserErrorReason::Incomplete {
                    kind: IncompleteKind::Decl(DeclKind::Import(ImportKind::MissingEndOfLine)),
                    cause,
                } = error.reason
                else {
                    panic!("Unexpected reason {:?}", error.reason);
                };

                assert_eq!(cause.start().offset, 7);
                assert_eq!(cause.end().offset, 10);
            }

            #[test]
            fn whitespace_only() {
                let input = InputSpan::new_extra("   \n", Config::default());
                let result = parse(input);

                let Err(nom::Err::Error(error)) = result else {
                    panic!("Expected error for whitespace only");
                };

                assert_eq!(error.error_offset, 0);
                assert!(matches!(
                    error.reason,
                    ParserErrorReason::Expect(ExpectKind::Decl)
                ));
            }

            #[test]
            fn comment_only() {
                let input = InputSpan::new_extra("# comment\n", Config::default());
                let result = parse(input);

                let Err(nom::Err::Error(error)) = result else {
                    panic!("Expected error for comment only");
                };

                assert_eq!(error.error_offset, 0);
                assert!(matches!(
                    error.reason,
                    ParserErrorReason::Expect(ExpectKind::Decl)
                ));
            }
        }

        mod use_error {
            use crate::error::reason::{
                DeclKind, ExpectKind, IncompleteKind, ParserErrorReason, UseKind,
            };

            use super::*;

            #[test]
            fn missing_use_keyword() {
                let input = InputSpan::new_extra("foo.bar as baz\n", Config::default());
                let result = parse(input);

                let Err(nom::Err::Error(error)) = result else {
                    panic!("Expected error for missing use keyword");
                };

                assert_eq!(error.error_offset, 0);

                assert!(matches!(
                    error.reason,
                    ParserErrorReason::Expect(ExpectKind::Decl)
                ));
            }

            #[test]
            fn missing_as_keyword() {
                let input = InputSpan::new_extra("use foo.bar baz\n", Config::default());
                let result = parse(input);

                // This should fail because 'baz' is not a valid continuation after a use declaration
                // The parser correctly parses "use foo.bar" but then expects a newline
                let Err(nom::Err::Failure(error)) = result else {
                    panic!("Expected error for invalid continuation after use declaration");
                };

                assert_eq!(error.error_offset, 12);

                let ParserErrorReason::Incomplete {
                    kind: IncompleteKind::Decl(DeclKind::Use(UseKind::MissingEndOfLine)),
                    cause,
                } = error.reason
                else {
                    panic!("Unexpected reason {:?}", error.reason);
                };

                // The cause should be the span of "foo.bar"
                assert_eq!(cause.start().offset, 4);
                assert_eq!(cause.end().offset, 11);
            }

            #[test]
            fn missing_alias() {
                let input = InputSpan::new_extra("use foo.bar as\n", Config::default());
                let result = parse(input);

                let Err(nom::Err::Failure(error)) = result else {
                    panic!("Unexpected result {result:?}");
                };

                assert_eq!(error.error_offset, 14);

                let ParserErrorReason::Incomplete {
                    kind: IncompleteKind::Decl(DeclKind::AsMissingAlias),
                    cause,
                } = error.reason
                else {
                    panic!("Unexpected reason {:?}", error.reason);
                };

                assert_eq!(cause.start().offset, 12);
                assert_eq!(cause.end().offset, 14);
            }

            #[test]
            fn invalid_path_identifier() {
                // TODO: Add context to this error (in error module): "invalid path identifier"
                let input = InputSpan::new_extra("use 123.bar as baz\n", Config::default());
                let result = parse(input);

                let Err(nom::Err::Failure(error)) = result else {
                    panic!("Unexpected result {result:?}");
                };

                assert_eq!(error.error_offset, 4);

                let ParserErrorReason::Incomplete {
                    kind: IncompleteKind::Decl(DeclKind::Use(UseKind::MissingModelInfo)),
                    cause,
                } = error.reason
                else {
                    panic!("Unexpected reason {:?}", error.reason);
                };

                assert_eq!(cause.start().offset, 0);
                assert_eq!(cause.end().offset, 3);
            }

            #[test]
            fn invalid_alias_identifier() {
                // TODO: Add context to this error (in error module): "invalid alias identifier"
                let input = InputSpan::new_extra("use foo.bar as 123\n", Config::default());
                let result = parse(input);

                let Err(nom::Err::Failure(error)) = result else {
                    panic!("Unexpected result {result:?}");
                };

                assert_eq!(error.error_offset, 15);

                let ParserErrorReason::Incomplete {
                    kind: IncompleteKind::Decl(DeclKind::AsMissingAlias),
                    cause,
                } = error.reason
                else {
                    panic!("Unexpected reason {:?}", error.reason);
                };

                assert_eq!(cause.start().offset, 12);
                assert_eq!(cause.end().offset, 14);
            }
        }

        mod model_info_error {
            use crate::error::reason::{DeclKind, IncompleteKind, ParserErrorReason};
            use crate::token::error::{ExpectKind, TokenErrorKind};

            use super::*;

            #[test]
            fn empty_path() {
                let input = InputSpan::new_extra("", Config::default());
                let result = model_info(input);

                let Err(nom::Err::Error(error)) = result else {
                    panic!("Unexpected result {result:?}");
                };

                assert_eq!(error.error_offset, 0);
                assert!(matches!(
                    error.reason,
                    ParserErrorReason::TokenError(TokenErrorKind::Expect(ExpectKind::Identifier))
                ));
            }

            #[test]
            fn invalid_first_identifier() {
                let input = InputSpan::new_extra("123.bar", Config::default());
                let result = model_info(input);

                let Err(nom::Err::Error(error)) = result else {
                    panic!("Unexpected result {result:?}");
                };

                assert_eq!(error.error_offset, 0);
                assert!(matches!(
                    error.reason,
                    ParserErrorReason::TokenError(TokenErrorKind::Expect(ExpectKind::Identifier))
                ));
            }

            #[test]
            fn missing_subcomponent_after_dot() {
                let input = InputSpan::new_extra("foo.", Config::default());
                let result = model_info(input);

                let Err(nom::Err::Failure(error)) = result else {
                    panic!("Unexpected result {result:?}");
                };

                assert_eq!(error.error_offset, 4);

                let ParserErrorReason::Incomplete {
                    kind: IncompleteKind::Decl(DeclKind::ModelMissingSubcomponent),
                    cause,
                } = error.reason
                else {
                    panic!("Unexpected reason {:?}", error.reason);
                };

                assert_eq!(cause.start().offset, 3);
                assert_eq!(cause.end().offset, 4);
            }

            #[test]
            fn invalid_subcomponent_after_dot() {
                let input = InputSpan::new_extra("foo.123", Config::default());
                let result = model_info(input);

                let Err(nom::Err::Failure(error)) = result else {
                    panic!("Unexpected result {result:?}");
                };

                assert_eq!(error.error_offset, 4);

                let ParserErrorReason::Incomplete {
                    kind: IncompleteKind::Decl(DeclKind::ModelMissingSubcomponent),
                    cause,
                } = error.reason
                else {
                    panic!("Unexpected reason {:?}", error.reason);
                };

                assert_eq!(cause.start().offset, 3);
                assert_eq!(cause.end().offset, 4);
            }

            #[test]
            fn dot_at_end() {
                let input = InputSpan::new_extra("foo.bar.", Config::default());
                let result = model_info(input);

                let Err(nom::Err::Failure(error)) = result else {
                    panic!("Unexpected result {result:?}");
                };

                assert_eq!(error.error_offset, 8);

                let ParserErrorReason::Incomplete {
                    kind: IncompleteKind::Decl(DeclKind::ModelMissingSubcomponent),
                    cause,
                } = error.reason
                else {
                    panic!("Unexpected reason {:?}", error.reason);
                };

                assert_eq!(cause.start().offset, 7);
                assert_eq!(cause.end().offset, 8);
            }
        }

        mod general_error {
            use crate::error::reason::{ExpectKind, IncompleteKind, ParserErrorReason};

            use super::*;

            #[test]
            fn no_valid_declaration() {
                let input = InputSpan::new_extra("invalid syntax\n", Config::default());
                let result = parse(input);

                let Err(nom::Err::Error(error)) = result else {
                    panic!("Unexpected result {result:?}");
                };

                assert_eq!(error.error_offset, 0);
                assert!(matches!(
                    error.reason,
                    ParserErrorReason::Expect(ExpectKind::Decl)
                ));
            }

            #[test]
            fn partial_keyword() {
                let input = InputSpan::new_extra("impor\n", Config::default());
                let result = parse(input);

                let Err(nom::Err::Error(error)) = result else {
                    panic!("Unexpected result {result:?}");
                };

                assert_eq!(error.error_offset, 0);
                assert!(matches!(
                    error.reason,
                    ParserErrorReason::Expect(ExpectKind::Decl)
                ));
            }

            #[test]
            fn wrong_keyword() {
                let input = InputSpan::new_extra("export foo\n", Config::default());
                let result = parse(input);

                let Err(nom::Err::Error(error)) = result else {
                    panic!("Unexpected result {result:?}");
                };

                assert_eq!(error.error_offset, 0);
                assert!(matches!(
                    error.reason,
                    ParserErrorReason::Expect(ExpectKind::Decl)
                ));
            }

            #[test]
            fn mixed_case_keywords() {
                let input = InputSpan::new_extra("Import foo\n", Config::default());
                let result = parse(input);

                let Err(nom::Err::Error(error)) = result else {
                    panic!("Unexpected result {result:?}");
                };

                assert_eq!(error.error_offset, 0);
                assert!(matches!(
                    error.reason,
                    ParserErrorReason::Expect(ExpectKind::Decl)
                ));
            }

            #[test]
            fn symbols_only() {
                let input = InputSpan::new_extra("+++---\n", Config::default());
                let result = parse(input);

                let Err(nom::Err::Error(error)) = result else {
                    panic!("Unexpected result {result:?}");
                };

                assert_eq!(error.error_offset, 0);
                assert!(matches!(
                    error.reason,
                    ParserErrorReason::Expect(ExpectKind::Decl)
                ));
            }

            #[test]
            fn numbers_only() {
                let input = InputSpan::new_extra("123 456\n", Config::default());
                let result = parse(input);

                let Err(nom::Err::Error(error)) = result else {
                    panic!("Unexpected result {result:?}");
                };

                assert_eq!(error.error_offset, 0);
                assert!(matches!(
                    error.reason,
                    ParserErrorReason::Expect(ExpectKind::Decl)
                ));
            }

            #[test]
            fn parse_complete_with_remaining_input() {
                let input = InputSpan::new_extra("import foo\nrest", Config::default());
                let result = parse_complete(input);

                let Err(nom::Err::Error(error)) = result else {
                    panic!("Unexpected result {result:?}");
                };

                assert_eq!(error.error_offset, 11);
                assert!(matches!(error.reason, ParserErrorReason::UnexpectedToken));
            }

            #[test]
            fn use_decl_with_unclosed_bracket_empty() {
                let input = InputSpan::new_extra("use foo with [\n", Config::default());
                let result = parse(input);

                let Err(nom::Err::Failure(error)) = result else {
                    panic!("Unexpected result {result:?}");
                };

                assert_eq!(error.error_offset, 15);

                let ParserErrorReason::Incomplete {
                    kind: IncompleteKind::UnclosedBracket,
                    cause,
                } = error.reason
                else {
                    panic!("Unexpected reason {:?}", error.reason);
                };

                assert_eq!(cause.start().offset, 13);
                assert_eq!(cause.end().offset, 14);
            }

            #[test]
            fn use_decl_with_unclosed_bracket_single_submodel() {
                let input = InputSpan::new_extra("use foo with [bar\n", Config::default());
                let result = parse(input);

                let Err(nom::Err::Failure(error)) = result else {
                    panic!("Unexpected result {result:?}");
                };

                assert_eq!(error.error_offset, 18);

                let ParserErrorReason::Incomplete {
                    kind: IncompleteKind::UnclosedBracket,
                    cause,
                } = error.reason
                else {
                    panic!("Unexpected reason {:?}", error.reason);
                };

                assert_eq!(cause.start().offset, 13);
                assert_eq!(cause.end().offset, 14);
            }

            #[test]
            fn use_decl_with_unclosed_bracket_multiple_submodels() {
                let input = InputSpan::new_extra("use foo with [bar, baz\n", Config::default());
                let result = parse(input);

                let Err(nom::Err::Failure(error)) = result else {
                    panic!("Unexpected result {result:?}");
                };

                assert_eq!(error.error_offset, 23);

                let ParserErrorReason::Incomplete {
                    kind: IncompleteKind::UnclosedBracket,
                    cause,
                } = error.reason
                else {
                    panic!("Unexpected reason {:?}", error.reason);
                };

                assert_eq!(cause.start().offset, 13);
                assert_eq!(cause.end().offset, 14);
            }

            #[test]
            fn use_decl_with_unclosed_bracket_with_trailing_comma() {
                let input = InputSpan::new_extra("use foo with [bar, baz,\n", Config::default());
                let result = parse(input);

                let Err(nom::Err::Failure(error)) = result else {
                    panic!("Unexpected result {result:?}");
                };

                assert_eq!(error.error_offset, 24);

                let ParserErrorReason::Incomplete {
                    kind: IncompleteKind::UnclosedBracket,
                    cause,
                } = error.reason
                else {
                    panic!("Unexpected reason {:?}", error.reason);
                };

                assert_eq!(cause.start().offset, 13);
                assert_eq!(cause.end().offset, 14);
            }

            #[test]
            fn use_decl_with_unclosed_bracket_with_subcomponents() {
                let input =
                    InputSpan::new_extra("use foo with [bar.qux, baz.quux\n", Config::default());
                let result = parse(input);

                let Err(nom::Err::Failure(error)) = result else {
                    panic!("Unexpected result {result:?}");
                };

                assert_eq!(error.error_offset, 32);

                let ParserErrorReason::Incomplete {
                    kind: IncompleteKind::UnclosedBracket,
                    cause,
                } = error.reason
                else {
                    panic!("Unexpected reason {:?}", error.reason);
                };

                assert_eq!(cause.start().offset, 13);
                assert_eq!(cause.end().offset, 14);
            }

            #[test]
            fn use_decl_with_unclosed_bracket_with_aliases() {
                let input = InputSpan::new_extra(
                    "use foo with [bar as baz, qux as quux\n",
                    Config::default(),
                );
                let result = parse(input);

                let Err(nom::Err::Failure(error)) = result else {
                    panic!("Unexpected result {result:?}");
                };

                assert_eq!(error.error_offset, 38);

                let ParserErrorReason::Incomplete {
                    kind: IncompleteKind::UnclosedBracket,
                    cause,
                } = error.reason
                else {
                    panic!("Unexpected reason {:?}", error.reason);
                };

                assert_eq!(cause.start().offset, 13);
                assert_eq!(cause.end().offset, 14);
            }

            #[test]
            fn use_decl_with_unclosed_bracket_with_model_alias() {
                let input =
                    InputSpan::new_extra("use foo as bar with [qux, baz\n", Config::default());
                let result = parse(input);

                let Err(nom::Err::Failure(error)) = result else {
                    panic!("Unexpected result {result:?}");
                };

                assert_eq!(error.error_offset, 30);

                let ParserErrorReason::Incomplete {
                    kind: IncompleteKind::UnclosedBracket,
                    cause,
                } = error.reason
                else {
                    panic!("Unexpected reason {:?}", error.reason);
                };

                assert_eq!(cause.start().offset, 20);
                assert_eq!(cause.end().offset, 21);
            }

            #[test]
            fn use_decl_with_unclosed_bracket_with_newlines() {
                let input = InputSpan::new_extra("use foo with [\nbar,\nbaz\n", Config::default());
                let result = parse(input);

                let Err(nom::Err::Failure(error)) = result else {
                    panic!("Unexpected result {result:?}");
                };

                assert_eq!(error.error_offset, 24);

                let ParserErrorReason::Incomplete {
                    kind: IncompleteKind::UnclosedBracket,
                    cause,
                } = error.reason
                else {
                    panic!("Unexpected reason {:?}", error.reason);
                };

                assert_eq!(cause.start().offset, 13);
                assert_eq!(cause.end().offset, 14);
            }

            #[test]
            fn use_decl_with_unclosed_bracket_with_complex_path() {
                let input = InputSpan::new_extra(
                    "use utils/math.trigonometry as trig with [sin, cos as cosine\n",
                    Config::default(),
                );
                let result = parse(input);

                let Err(nom::Err::Failure(error)) = result else {
                    panic!("Unexpected result {result:?}");
                };

                assert_eq!(error.error_offset, 61);

                let ParserErrorReason::Incomplete {
                    kind: IncompleteKind::UnclosedBracket,
                    cause,
                } = error.reason
                else {
                    panic!("Unexpected reason {:?}", error.reason);
                };

                assert_eq!(cause.start().offset, 41);
                assert_eq!(cause.end().offset, 42);
            }
        }
    }
}
