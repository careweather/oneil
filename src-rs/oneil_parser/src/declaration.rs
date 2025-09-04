//! Parser for declarations in an Oneil program.

use nom::{
    Parser as _,
    branch::alt,
    combinator::{all_consuming, opt},
    multi::many0,
};

use oneil_ast::{
    AstSpan as AstSpan,
    declaration::{
        Decl, DeclNode, Import, ModelInfo, ModelInfoNode, ModelKind, SubmodelList,
        SubmodelListNode, UseModel,
    },
    naming::{Directory, DirectoryNode, Identifier, IdentifierNode},
    node::Node,
};

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
    util::{Result, InputSpan},
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
///
/// An import declaration has the format: `import path` followed by a newline.
/// The path is a simple identifier that represents the module or file to import.
///
/// Examples:
/// - `import foo`
/// - `import my_module`
/// - `import utils`
///
/// The parser requires:
/// - The `import` keyword
/// - A valid identifier as the import path
/// - A newline to terminate the declaration
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns a declaration node containing the parsed import, or an error if
/// the import declaration is malformed.
fn import_decl(input: InputSpan<'_>) -> Result<'_, DeclNode, ParserError> {
    let (rest, import_token) = import.convert_errors().parse(input)?;

    let (rest, import_path_token) = identifier
        .or_fail_with(ParserError::import_missing_path(&import_token))
        .parse(rest)?;

    let (rest, end_of_line_token) = end_of_line
        .or_fail_with(ParserError::import_missing_end_of_line(&import_path_token))
        .parse(rest)?;

    let span =
        AstSpan::calc_span_with_whitespace(&import_token, &import_path_token, &end_of_line_token);

    let import_path = Node::new(&import_path_token, import_path_token.lexeme().to_string());
    let import_node = Node::new(&span, Import::new(import_path));

    let decl_node = Node::new(&span, Decl::Import(import_node));

    Ok((rest, decl_node))
}

/// Parses a use declaration
///
/// A use declaration has the format: `use path [inputs] as alias` followed by a newline.
/// This declaration imports a module or model and gives it a local alias.
///
/// Examples:
/// - `use foo.bar as baz`
/// - `use utils.math(x=1, y=2) as calculator`
/// - `use my_module as local_name`
///
/// The parser requires:
/// - The `use` keyword
/// - A model path (e.g., "foo.bar")
/// - Optional model inputs in parentheses
/// - The `as` keyword
/// - A valid identifier as the alias
/// - A newline to terminate the declaration
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns a declaration node containing the parsed use declaration, or an error if
/// the declaration is malformed.
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
        .or_fail_with(ParserError::use_missing_model_info(&keyword_token))
        .parse(rest)?;

    let (rest, submodel_list) = opt(|input| {
        let (rest, _with_token) = with.convert_errors().parse(input)?;
        submodel_list(rest)
    })
    .parse(rest)?;

    let final_span = match &submodel_list {
        Some(submodel_list) => submodel_list.node_span(),
        None => model_info.node_span(),
    };

    let (rest, end_of_line_token) = end_of_line
        .or_fail_with(ParserError::use_missing_end_of_line(&final_span))
        .parse(rest)?;

    let span = AstSpan::calc_span_with_whitespace(&keyword_token, &final_span, &end_of_line_token);

    let use_model_node = Node::new(
        &span,
        UseModel::new(directory_path, model_info, submodel_list, is_ref_only),
    );

    let decl_node = Node::new(&span, Decl::UseModel(use_model_node));

    Ok((rest, decl_node))
}

/// Parses a directory path in a model path
///
/// A directory path is a sequence of directory names separated by forward slashes.
/// The directory path is optional - if no directory path is present, an empty vector is returned.
///
/// Examples:
/// - `path/to/` (two directory names)
/// - `../foo/` (parent directory followed by identifier)
/// - `./bar/` (current directory followed by identifier)
/// - ` ` (empty path)
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns a vector of directory nodes representing the directory path.
/// Each directory node contains one of:
/// - `Directory::Name(String)` for an identifier
/// - `Directory::Current` for "."
/// - `Directory::Parent` for ".."
fn opt_directory_path(input: InputSpan<'_>) -> Result<'_, Vec<DirectoryNode>, ParserError> {
    many0(|input| {
        let (rest, directory_name) = directory_name(input)?;
        let (rest, _slash_token) = slash.convert_errors().parse(rest)?;
        Ok((rest, directory_name))
    })
    .parse(input)
}

/// Parses a directory name in a model path
///
/// A directory name can be one of:
/// - An identifier (e.g. "foo")
/// - A current directory marker (".")
/// - A parent directory marker ("..")
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns a directory node containing the parsed directory name.
/// The directory node will contain one of:
/// - `Directory::Name(String)` for an identifier
/// - `Directory::Current` for "."
/// - `Directory::Parent` for ".."
fn directory_name(input: InputSpan<'_>) -> Result<'_, DirectoryNode, ParserError> {
    let directory_name = |input| {
        let (rest, directory_name_token) = identifier.convert_errors().parse(input)?;
        let directory_name = Node::new(
            &directory_name_token,
            Directory::name(directory_name_token.lexeme().to_string()),
        );
        Ok((rest, directory_name))
    };

    let current_directory = |input| {
        let (rest, dot_token) = dot.convert_errors().parse(input)?;
        let current_directory = Node::new(&dot_token, Directory::current());
        Ok((rest, current_directory))
    };

    let parent_directory = |input| {
        let (rest, dot_dot_token) = dot_dot.convert_errors().parse(input)?;
        let parent_directory = Node::new(&dot_dot_token, Directory::parent());
        Ok((rest, parent_directory))
    };

    alt((directory_name, current_directory, parent_directory)).parse(input)
}

pub fn model_info(input: InputSpan<'_>) -> Result<'_, ModelInfoNode, ParserError> {
    let (rest, top_component) = identifier.convert_errors().parse(input)?;
    let top_component_ident = Identifier::new(top_component.lexeme().to_string());
    let top_component = Node::new(&top_component, top_component_ident);

    let (rest, subcomponents) = opt_subcomponents.parse(rest)?;
    let (rest, alias) = opt(as_alias).parse(rest)?;

    let last_span = match (subcomponents.last(), &alias) {
        (_, Some(alias)) => alias.node_span(),
        (Some(subcomponent), None) => subcomponent.node_span(),
        (None, None) => top_component.node_span(),
    };

    let model_info_span = AstSpan::calc_span(&top_component, &last_span);
    let model_info = ModelInfo::new(top_component, subcomponents, alias);
    Ok((rest, Node::new(&model_info_span, model_info)))
}

fn opt_subcomponents(input: InputSpan<'_>) -> Result<'_, Vec<IdentifierNode>, ParserError> {
    many0(|input| {
        let (rest, dot_token) = dot.convert_errors().parse(input)?;
        let (rest, subcomponent) = identifier
            .or_fail_with(ParserError::model_path_missing_subcomponent(&dot_token))
            .parse(rest)?;
        let subcomponent_node = Node::new(
            &subcomponent,
            Identifier::new(subcomponent.lexeme().to_string()),
        );
        Ok((rest, subcomponent_node))
    })
    .parse(input)
}

/// Parses an alias identifier after an `as` keyword.
///
/// This function parses the alias identifier that follows an `as` keyword in a use declaration.
/// It expects a valid identifier token after the `as` keyword.
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns an identifier node containing the parsed alias.
/// Returns an error if no valid identifier follows the `as` keyword.
///
/// # Examples
///
/// ```text
/// as foo  -> IdentifierNode("foo")
/// as      -> Error(MissingAlias)
/// as 123  -> Error(UnexpectedToken)
/// ```
fn as_alias(input: InputSpan<'_>) -> Result<'_, IdentifierNode, ParserError> {
    let (rest, as_token) = as_.convert_errors().parse(input)?;

    let (rest, alias) = identifier
        .or_fail_with(ParserError::as_missing_alias(&as_token))
        .parse(rest)?;
    let alias_node = Node::new(&alias, Identifier::new(alias.lexeme().to_string()));

    Ok((rest, alias_node))
}

/// Parses a list of submodels in a use declaration
///
/// A submodel list can be either a single submodel or multiple submodels enclosed in square brackets.
/// For multiple submodels, they are separated by commas and can have an optional trailing comma. The
/// trailing comma is allowed in order to make git diffs easier to read.
///
/// Examples:
/// - Single submodel: `foo.bar as baz`
/// - Multiple submodels: `[foo.bar as baz, qux.quux]`
/// - Multiple with trailing comma: `[foo.bar, baz.qux,]`
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns a vector of submodel nodes. For a single submodel, returns a vector of length 1.
/// For multiple submodels in brackets, returns a vector containing all parsed submodels.
/// Returns an error if the submodel list is malformed (e.g., unclosed brackets).
fn submodel_list(input: InputSpan<'_>) -> Result<'_, SubmodelListNode, ParserError> {
    let single_submodel = |input| {
        let (rest, submodel) = model_info.parse(input)?;
        let submodel_span = submodel.node_span();

        let submodel_list = SubmodelList::new(vec![submodel]);
        let submodel_list_node = Node::new(&submodel_span, submodel_list);

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
            .or_fail_with(ParserError::unclosed_bracket(&bracket_left_token))
            .parse(rest)?;

        let submodel_list = SubmodelList::new(submodel_list.unwrap_or_default());

        let submodel_list_span = AstSpan::calc_span(&bracket_left_token, &bracket_right_token);
        let submodel_list_node = Node::new(&submodel_list_span, submodel_list);

        Ok((rest, submodel_list_node))
    };

    alt((single_submodel, multiple_submodels)).parse(input)
}

/// Parses a parameter declaration by delegating to the parameter parser.
///
/// This function wraps the parameter parser to create a declaration node.
/// It handles the conversion from a parameter node to a declaration node
/// with proper span calculation.
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns a declaration node containing the parsed parameter.
fn parameter_decl(input: InputSpan<'_>) -> Result<'_, DeclNode, ParserError> {
    let (rest, parameter) = parse_parameter.parse(input)?;

    let span = AstSpan::from(&parameter);
    let decl_node = Node::new(&span, Decl::Parameter(parameter));

    Ok((rest, decl_node))
}

/// Parses a test declaration by delegating to the test parser.
///
/// This function wraps the test parser to create a declaration node.
/// It handles the conversion from a test node to a declaration node
/// with proper span calculation.
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns a declaration node containing the parsed test.
fn test_decl(input: InputSpan<'_>) -> Result<'_, DeclNode, ParserError> {
    let (rest, test) = parse_test.parse(input)?;

    let span = AstSpan::from(&test);
    let decl_node = Node::new(&span, Decl::Test(test));

    Ok((rest, decl_node))
}

#[cfg(test)]
#[allow(
    clippy::similar_names,
    reason = "tests make it clear what variable is being tested"
)]
mod tests {
    use super::*;
    use crate::Config;

    mod success_tests {
        use super::*;

        #[test]
        fn test_import_decl() {
            let input = InputSpan::new_extra("import foo\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::Import(import_node) => {
                    let import_path = import_node.path();

                    assert_eq!(import_path.node_value(), "foo");
                    assert_eq!(import_path.node_span(), AstSpan::new(7, 3, 0));
                }
                _ => panic!("Expected import declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_ref_decl() {
            let input = InputSpan::new_extra("ref foo\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model_info = use_model_node.model_info();
                    assert_eq!(use_model_info.top_component().as_str(), "foo");
                    assert_eq!(use_model_info.subcomponents().len(), 0);
                    assert_eq!(use_model_info.get_alias().as_str(), "foo");
                    assert_eq!(use_model_node.model_kind(), ModelKind::Reference);
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_decl() {
            let input = InputSpan::new_extra("use foo.bar as baz\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model_info = use_model_node.model_info();
                    assert_eq!(use_model_info.top_component().as_str(), "foo");
                    assert_eq!(use_model_info.subcomponents()[0].as_str(), "bar");
                    assert_eq!(use_model_info.get_alias().as_str(), "baz");
                    assert_eq!(use_model_node.model_kind(), ModelKind::Submodel);
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_decl_without_alias() {
            let input = InputSpan::new_extra("use foo.bar\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model_info = use_model_node.model_info();
                    assert_eq!(use_model_info.top_component().as_str(), "foo");
                    assert_eq!(use_model_info.subcomponents().len(), 1);
                    assert_eq!(use_model_info.subcomponents()[0].as_str(), "bar");
                    assert_eq!(use_model_info.get_alias().as_str(), "bar");
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_decl_simple_without_alias() {
            let input = InputSpan::new_extra("use foo\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model_info = use_model_node.model_info();
                    assert_eq!(use_model_info.top_component().as_str(), "foo");
                    assert_eq!(use_model_info.subcomponents().len(), 0);
                    assert_eq!(use_model_info.get_alias().as_str(), "foo");
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_parse_complete_import_success() {
            let input = InputSpan::new_extra("import foo\n", Config::default());
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::Import(import_node) => {
                    let import_path = import_node.path();
                    assert_eq!(import_path.node_value(), "foo");
                    assert_eq!(import_path.node_span(), AstSpan::new(7, 3, 0));
                }
                _ => panic!("Expected import declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_parse_complete_use_success() {
            let input = InputSpan::new_extra("use foo.bar as baz\n", Config::default());
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model_info = use_model_node.model_info();
                    assert_eq!(use_model_info.top_component().as_str(), "foo");
                    assert_eq!(use_model_info.subcomponents()[0].as_str(), "bar");
                    assert_eq!(use_model_info.get_alias().as_str(), "baz");
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_with_single_directory() {
            let input = InputSpan::new_extra("use utils/math as calculator\n", Config::default());
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model_info = use_model_node.model_info();
                    assert_eq!(use_model_info.top_component().as_str(), "math");
                    assert_eq!(use_model_info.subcomponents().len(), 0);
                    assert_eq!(use_model_info.get_alias().as_str(), "calculator");

                    // Check directory path
                    assert_eq!(use_model_node.directory_path().len(), 1);
                    assert_eq!(
                        use_model_node.directory_path()[0].node_value().as_str(),
                        "utils"
                    );
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_with_single_directory_without_alias() {
            let input = InputSpan::new_extra("use utils/math\n", Config::default());
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model_info = use_model_node.model_info();
                    assert_eq!(use_model_info.top_component().as_str(), "math");
                    assert_eq!(use_model_info.subcomponents().len(), 0);
                    assert_eq!(use_model_info.get_alias().as_str(), "math");

                    // Check directory path
                    assert_eq!(use_model_node.directory_path().len(), 1);
                    assert_eq!(
                        use_model_node.directory_path()[0].node_value().as_str(),
                        "utils"
                    );
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_with_multiple_directories() {
            let input = InputSpan::new_extra(
                "use models/physics/mechanics as dynamics\n",
                Config::default(),
            );
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model_info = use_model_node.model_info();
                    assert_eq!(use_model_info.top_component().as_str(), "mechanics");
                    assert_eq!(use_model_info.subcomponents().len(), 0);
                    assert_eq!(use_model_info.get_alias().as_str(), "dynamics");

                    // Check directory path
                    assert_eq!(use_model_node.directory_path().len(), 2);
                    assert_eq!(
                        use_model_node.directory_path()[0].node_value().as_str(),
                        "models"
                    );
                    assert_eq!(
                        use_model_node.directory_path()[1].node_value().as_str(),
                        "physics"
                    );
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_with_directory_and_subcomponents() {
            let input = InputSpan::new_extra("use utils/math.trigonometry as trig\n", Config::default());
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model_info = use_model_node.model_info();
                    assert_eq!(use_model_info.top_component().as_str(), "math");
                    assert_eq!(use_model_info.subcomponents().len(), 1);
                    assert_eq!(use_model_info.subcomponents()[0].as_str(), "trigonometry");
                    assert_eq!(use_model_info.get_alias().as_str(), "trig");

                    // Check directory path
                    assert_eq!(use_model_node.directory_path().len(), 1);
                    assert_eq!(
                        use_model_node.directory_path()[0].node_value().as_str(),
                        "utils"
                    );
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_with_current_directory() {
            let input = InputSpan::new_extra("use ./local_model as local\n", Config::default());
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model_info = use_model_node.model_info();
                    assert_eq!(use_model_info.top_component().as_str(), "local_model");
                    assert_eq!(use_model_info.subcomponents().len(), 0);
                    assert_eq!(use_model_info.get_alias().as_str(), "local");

                    // Check directory path
                    assert_eq!(use_model_node.directory_path().len(), 1);
                    assert_eq!(
                        use_model_node.directory_path()[0].node_value().as_str(),
                        "."
                    );
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_with_parent_directory() {
            let input = InputSpan::new_extra("use ../parent_model as parent\n", Config::default());
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model_info = use_model_node.model_info();
                    assert_eq!(use_model_info.top_component().as_str(), "parent_model");
                    assert_eq!(use_model_info.subcomponents().len(), 0);
                    assert_eq!(use_model_info.get_alias().as_str(), "parent");

                    // Check directory path
                    assert_eq!(use_model_node.directory_path().len(), 1);
                    assert_eq!(
                        use_model_node.directory_path()[0].node_value().as_str(),
                        ".."
                    );
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_with_mixed_directory_types() {
            let input = InputSpan::new_extra(
                "use ../shared/./utils/math as shared_math\n",
                Config::default(),
            );
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model_info = use_model_node.model_info();
                    assert_eq!(use_model_info.top_component().as_str(), "math");
                    assert_eq!(use_model_info.subcomponents().len(), 0);
                    assert_eq!(use_model_info.get_alias().as_str(), "shared_math");

                    // Check directory path
                    assert_eq!(use_model_node.directory_path().len(), 4);
                    assert_eq!(
                        use_model_node.directory_path()[0].node_value().as_str(),
                        ".."
                    );
                    assert_eq!(
                        use_model_node.directory_path()[1].node_value().as_str(),
                        "shared"
                    );
                    assert_eq!(
                        use_model_node.directory_path()[2].node_value().as_str(),
                        "."
                    );
                    assert_eq!(
                        use_model_node.directory_path()[3].node_value().as_str(),
                        "utils"
                    );
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_with_complex_path_and_subcomponents() {
            let input = InputSpan::new_extra(
                "use models/physics/mechanics.rotational.dynamics as rotation\n",
                Config::default(),
            );
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model_info = use_model_node.model_info();
                    assert_eq!(use_model_info.top_component().as_str(), "mechanics");
                    assert_eq!(use_model_info.subcomponents().len(), 2);
                    assert_eq!(use_model_info.subcomponents()[0].as_str(), "rotational");
                    assert_eq!(use_model_info.subcomponents()[1].as_str(), "dynamics");
                    assert_eq!(use_model_info.get_alias().as_str(), "rotation");

                    // Check directory path
                    assert_eq!(use_model_node.directory_path().len(), 2);
                    assert_eq!(
                        use_model_node.directory_path()[0].node_value().as_str(),
                        "models"
                    );
                    assert_eq!(
                        use_model_node.directory_path()[1].node_value().as_str(),
                        "physics"
                    );
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_directory_name_parsing() {
            // Test parent directory
            let input = InputSpan::new_extra("..", Config::default());
            let (rest, dir) = directory_name(input).expect("should parse parent directory");
            assert_eq!(dir.node_value().as_str(), "..");
            assert_eq!(rest.fragment(), &"");

            // Test current directory
            let input = InputSpan::new_extra(".", Config::default());
            let (rest, dir) = directory_name(input).expect("should parse current directory");
            assert_eq!(dir.node_value().as_str(), ".");
            assert_eq!(rest.fragment(), &"");

            // Test regular directory name
            let input = InputSpan::new_extra("foo", Config::default());
            let (rest, dir) = directory_name(input).expect("should parse regular directory name");
            assert_eq!(dir.node_value().as_str(), "foo");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_mixed_directory_path_parsing() {
            let input = InputSpan::new_extra("../shared/./utils/", Config::default());
            let (_rest, directory_path) =
                opt_directory_path(input).expect("should parse mixed directory path");

            assert_eq!(directory_path.len(), 4);
            assert_eq!(directory_path[0].node_value(), &Directory::Parent);
            assert_eq!(
                directory_path[1].node_value(),
                &Directory::Name("shared".to_string())
            );
            assert_eq!(directory_path[2].node_value(), &Directory::Current);
            assert_eq!(
                directory_path[3].node_value(),
                &Directory::Name("utils".to_string())
            );
        }

        #[test]
        fn test_use_decl_with_single_submodel() {
            let input = InputSpan::new_extra("use foo with bar\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model_info = use_model_node.model_info();
                    assert_eq!(use_model_info.top_component().as_str(), "foo");
                    assert_eq!(use_model_info.subcomponents().len(), 0);
                    assert_eq!(use_model_info.get_alias().as_str(), "foo");

                    // Check submodels
                    let submodels = use_model_node.submodels().expect("should have submodels");
                    assert_eq!(submodels.len(), 1);

                    let submodel = &submodels[0];
                    assert_eq!(submodel.node_value().top_component().as_str(), "bar");
                    assert_eq!(submodel.node_value().subcomponents().len(), 0);
                    assert_eq!(submodel.node_value().get_alias().as_str(), "bar");
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_decl_with_single_submodel_with_alias() {
            let input = InputSpan::new_extra("use foo with bar as baz\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model_info = use_model_node.model_info();
                    assert_eq!(use_model_info.top_component().as_str(), "foo");
                    assert_eq!(use_model_info.subcomponents().len(), 0);
                    assert_eq!(use_model_info.get_alias().as_str(), "foo");

                    // Check submodels
                    let submodels = use_model_node.submodels().expect("should have submodels");
                    assert_eq!(submodels.len(), 1);

                    let submodel = &submodels[0];
                    assert_eq!(submodel.node_value().top_component().as_str(), "bar");
                    assert_eq!(submodel.node_value().subcomponents().len(), 0);
                    assert_eq!(submodel.node_value().get_alias().as_str(), "baz");
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_decl_with_single_submodel_with_subcomponents() {
            let input = InputSpan::new_extra("use foo with bar.qux\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model_info = use_model_node.model_info();
                    assert_eq!(use_model_info.top_component().as_str(), "foo");
                    assert_eq!(use_model_info.subcomponents().len(), 0);
                    assert_eq!(use_model_info.get_alias().as_str(), "foo");

                    // Check submodels
                    let submodels = use_model_node.submodels().expect("should have submodels");
                    assert_eq!(submodels.len(), 1);

                    let submodel = &submodels[0];
                    assert_eq!(submodel.node_value().top_component().as_str(), "bar");
                    assert_eq!(submodel.node_value().subcomponents().len(), 1);
                    assert_eq!(submodel.node_value().subcomponents()[0].as_str(), "qux");
                    assert_eq!(submodel.node_value().get_alias().as_str(), "qux");
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_decl_with_multiple_submodels() {
            let input = InputSpan::new_extra("use foo with [bar, qux]\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model_info = use_model_node.model_info();
                    assert_eq!(use_model_info.top_component().as_str(), "foo");
                    assert_eq!(use_model_info.subcomponents().len(), 0);
                    assert_eq!(use_model_info.get_alias().as_str(), "foo");

                    // Check submodels
                    let submodels = use_model_node.submodels().expect("should have submodels");
                    assert_eq!(submodels.len(), 2);

                    let submodel1 = &submodels[0];
                    assert_eq!(submodel1.node_value().top_component().as_str(), "bar");
                    assert_eq!(submodel1.node_value().subcomponents().len(), 0);
                    assert_eq!(submodel1.node_value().get_alias().as_str(), "bar");

                    let submodel2 = &submodels[1];
                    assert_eq!(submodel2.node_value().top_component().as_str(), "qux");
                    assert_eq!(submodel2.node_value().subcomponents().len(), 0);
                    assert_eq!(submodel2.node_value().get_alias().as_str(), "qux");
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_decl_with_multiple_submodels_with_aliases() {
            let input = InputSpan::new_extra(
                "use foo with [bar as baz, qux as quux]\n",
                Config::default(),
            );
            let (rest, decl) = parse(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model_info = use_model_node.model_info();
                    assert_eq!(use_model_info.top_component().as_str(), "foo");
                    assert_eq!(use_model_info.subcomponents().len(), 0);
                    assert_eq!(use_model_info.get_alias().as_str(), "foo");

                    // Check submodels
                    let submodels = use_model_node.submodels().expect("should have submodels");
                    assert_eq!(submodels.len(), 2);

                    let submodel1 = &submodels[0];
                    assert_eq!(submodel1.node_value().top_component().as_str(), "bar");
                    assert_eq!(submodel1.node_value().subcomponents().len(), 0);
                    assert_eq!(submodel1.node_value().get_alias().as_str(), "baz");

                    let submodel2 = &submodels[1];
                    assert_eq!(submodel2.node_value().top_component().as_str(), "qux");
                    assert_eq!(submodel2.node_value().subcomponents().len(), 0);
                    assert_eq!(submodel2.node_value().get_alias().as_str(), "quux");
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_decl_with_multiple_submodels_with_subcomponents() {
            let input =
                InputSpan::new_extra("use foo with [bar.qux, baz.quux.quuz]\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model_info = use_model_node.model_info();
                    assert_eq!(use_model_info.top_component().as_str(), "foo");
                    assert_eq!(use_model_info.subcomponents().len(), 0);
                    assert_eq!(use_model_info.get_alias().as_str(), "foo");

                    // Check submodels
                    let submodels = use_model_node.submodels().expect("should have submodels");
                    assert_eq!(submodels.len(), 2);

                    let submodel1 = &submodels[0];
                    assert_eq!(submodel1.node_value().top_component().as_str(), "bar");
                    assert_eq!(submodel1.node_value().subcomponents().len(), 1);
                    assert_eq!(submodel1.node_value().subcomponents()[0].as_str(), "qux");
                    assert_eq!(submodel1.node_value().get_alias().as_str(), "qux");

                    let submodel2 = &submodels[1];
                    assert_eq!(submodel2.node_value().top_component().as_str(), "baz");
                    assert_eq!(submodel2.node_value().subcomponents().len(), 2);
                    assert_eq!(submodel2.node_value().subcomponents()[0].as_str(), "quux");
                    assert_eq!(submodel2.node_value().subcomponents()[1].as_str(), "quuz");
                    assert_eq!(submodel2.node_value().get_alias().as_str(), "quuz");
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_decl_with_multiple_submodels_with_trailing_comma() {
            let input = InputSpan::new_extra("use foo with [bar, qux,]\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model_info = use_model_node.model_info();
                    assert_eq!(use_model_info.top_component().as_str(), "foo");
                    assert_eq!(use_model_info.subcomponents().len(), 0);
                    assert_eq!(use_model_info.get_alias().as_str(), "foo");

                    // Check submodels
                    let submodels = use_model_node.submodels().expect("should have submodels");
                    assert_eq!(submodels.len(), 2);

                    let submodel1 = &submodels[0];
                    assert_eq!(submodel1.node_value().top_component().as_str(), "bar");
                    assert_eq!(submodel1.node_value().subcomponents().len(), 0);
                    assert_eq!(submodel1.node_value().get_alias().as_str(), "bar");

                    let submodel2 = &submodels[1];
                    assert_eq!(submodel2.node_value().top_component().as_str(), "qux");
                    assert_eq!(submodel2.node_value().subcomponents().len(), 0);
                    assert_eq!(submodel2.node_value().get_alias().as_str(), "qux");
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_decl_with_empty_submodel_list() {
            let input = InputSpan::new_extra("use foo with []\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model_info = use_model_node.model_info();
                    assert_eq!(use_model_info.top_component().as_str(), "foo");
                    assert_eq!(use_model_info.subcomponents().len(), 0);
                    assert_eq!(use_model_info.get_alias().as_str(), "foo");

                    // Check submodels - should be empty
                    let submodels = use_model_node.submodels().expect("should have submodels");
                    assert_eq!(submodels.len(), 0);
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_decl_with_model_alias_and_submodels() {
            let input = InputSpan::new_extra("use foo as bar with [qux, baz]\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model_info = use_model_node.model_info();
                    assert_eq!(use_model_info.top_component().as_str(), "foo");
                    assert_eq!(use_model_info.subcomponents().len(), 0);
                    assert_eq!(use_model_info.get_alias().as_str(), "bar");

                    // Check submodels
                    let submodels = use_model_node.submodels().expect("should have submodels");
                    assert_eq!(submodels.len(), 2);

                    let submodel1 = &submodels[0];
                    assert_eq!(submodel1.node_value().top_component().as_str(), "qux");
                    assert_eq!(submodel1.node_value().subcomponents().len(), 0);
                    assert_eq!(submodel1.node_value().get_alias().as_str(), "qux");

                    let submodel2 = &submodels[1];
                    assert_eq!(submodel2.node_value().top_component().as_str(), "baz");
                    assert_eq!(submodel2.node_value().subcomponents().len(), 0);
                    assert_eq!(submodel2.node_value().get_alias().as_str(), "baz");
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_decl_with_complex_path_and_submodels() {
            let input = InputSpan::new_extra(
                "use utils/math.trigonometry as trig with [sin, cos as cosine]\n",
                Config::default(),
            );
            let (rest, decl) = parse(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model_info = use_model_node.model_info();
                    assert_eq!(use_model_info.top_component().as_str(), "math");
                    assert_eq!(use_model_info.subcomponents().len(), 1);
                    assert_eq!(use_model_info.subcomponents()[0].as_str(), "trigonometry");
                    assert_eq!(use_model_info.get_alias().as_str(), "trig");

                    // Check directory path
                    assert_eq!(use_model_node.directory_path().len(), 1);
                    assert_eq!(
                        use_model_node.directory_path()[0].node_value().as_str(),
                        "utils"
                    );

                    // Check submodels
                    let submodels = use_model_node.submodels().expect("should have submodels");
                    assert_eq!(submodels.len(), 2);

                    let submodel1 = &submodels[0];
                    assert_eq!(submodel1.node_value().top_component().as_str(), "sin");
                    assert_eq!(submodel1.node_value().subcomponents().len(), 0);
                    assert_eq!(submodel1.node_value().get_alias().as_str(), "sin");

                    let submodel2 = &submodels[1];
                    assert_eq!(submodel2.node_value().top_component().as_str(), "cos");
                    assert_eq!(submodel2.node_value().subcomponents().len(), 0);
                    assert_eq!(submodel2.node_value().get_alias().as_str(), "cosine");
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_decl_with_submodels_and_newlines() {
            let input = InputSpan::new_extra("use foo with [\nbar,\nqux\n]\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model_info = use_model_node.model_info();
                    assert_eq!(use_model_info.top_component().as_str(), "foo");
                    assert_eq!(use_model_info.subcomponents().len(), 0);
                    assert_eq!(use_model_info.get_alias().as_str(), "foo");

                    // Check submodels
                    let submodels = use_model_node.submodels().expect("should have submodels");
                    assert_eq!(submodels.len(), 2);

                    let submodel1 = &submodels[0];
                    assert_eq!(submodel1.node_value().top_component().as_str(), "bar");
                    assert_eq!(submodel1.node_value().subcomponents().len(), 0);
                    assert_eq!(submodel1.node_value().get_alias().as_str(), "bar");

                    let submodel2 = &submodels[1];
                    assert_eq!(submodel2.node_value().top_component().as_str(), "qux");
                    assert_eq!(submodel2.node_value().subcomponents().len(), 0);
                    assert_eq!(submodel2.node_value().get_alias().as_str(), "qux");
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }
    }

    mod error_tests {
        use super::*;

        mod import_error_tests {
            use crate::error::reason::{
                DeclKind, ExpectKind, ImportKind, IncompleteKind, ParserErrorReason,
            };

            use super::*;

            #[test]
            fn test_empty_input() {
                let input = InputSpan::new_extra("", Config::default());
                let result = parse(input);
                match result {
                    Err(nom::Err::Error(error)) => {
                        assert_eq!(error.error_offset, 0);
                        assert!(matches!(
                            error.reason,
                            ParserErrorReason::Expect(ExpectKind::Decl)
                        ));
                    }
                    _ => panic!("Expected error for empty input"),
                }
            }

            #[test]
            fn test_missing_import_keyword() {
                let input = InputSpan::new_extra("foo\n", Config::default());
                let result = parse(input);
                match result {
                    Err(nom::Err::Error(error)) => {
                        assert_eq!(error.error_offset, 0);
                        assert!(matches!(
                            error.reason,
                            ParserErrorReason::Expect(ExpectKind::Decl)
                        ));
                    }
                    _ => panic!("Expected error for missing import keyword"),
                }
            }

            #[test]
            fn test_missing_path() {
                let input = InputSpan::new_extra("import\n", Config::default());
                let result = parse(input);
                let expected_import_span = AstSpan::new(0, 6, 0);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 6);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind:
                                    IncompleteKind::Decl(DeclKind::Import(ImportKind::MissingPath)),
                                cause,
                            } => {
                                assert_eq!(cause, expected_import_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {result:?}"),
                }
            }

            #[test]
            fn test_invalid_path_identifier() {
                let input = InputSpan::new_extra("import 123\n", Config::default());
                let result = parse(input);
                let expected_import_span = AstSpan::new(0, 6, 1);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 7);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind:
                                    IncompleteKind::Decl(DeclKind::Import(ImportKind::MissingPath)),
                                cause,
                            } => {
                                assert_eq!(cause, expected_import_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {result:?}"),
                }
            }

            #[test]
            fn test_path_with_missing_end_of_line() {
                let input = InputSpan::new_extra("import foo@bar\n", Config::default());
                let result = parse(input);
                let expected_foo_span = AstSpan::new(7, 3, 0);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 10);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind:
                                    IncompleteKind::Decl(DeclKind::Import(ImportKind::MissingEndOfLine)),
                                cause,
                            } => {
                                assert_eq!(cause, expected_foo_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {result:?}"),
                }
            }

            #[test]
            fn test_whitespace_only() {
                let input = InputSpan::new_extra("   \n", Config::default());
                let result = parse(input);
                match result {
                    Err(nom::Err::Error(error)) => {
                        assert_eq!(error.error_offset, 0);
                        assert!(matches!(
                            error.reason,
                            ParserErrorReason::Expect(ExpectKind::Decl)
                        ));
                    }
                    _ => panic!("Expected error for whitespace only"),
                }
            }

            #[test]
            fn test_comment_only() {
                let input = InputSpan::new_extra("# comment\n", Config::default());
                let result = parse(input);
                match result {
                    Err(nom::Err::Error(error)) => {
                        assert_eq!(error.error_offset, 0);
                        assert!(matches!(
                            error.reason,
                            ParserErrorReason::Expect(ExpectKind::Decl)
                        ));
                    }
                    _ => panic!("Expected error for comment only"),
                }
            }
        }

        mod use_error_tests {
            use crate::error::reason::{
                DeclKind, ExpectKind, IncompleteKind, ParserErrorReason, UseKind,
            };

            use super::*;

            #[test]
            fn test_missing_use_keyword() {
                let input = InputSpan::new_extra("foo.bar as baz\n", Config::default());
                let result = parse(input);
                match result {
                    Err(nom::Err::Error(error)) => {
                        assert_eq!(error.error_offset, 0);
                        assert!(matches!(
                            error.reason,
                            ParserErrorReason::Expect(ExpectKind::Decl)
                        ));
                    }
                    _ => panic!("Expected error for missing use keyword"),
                }
            }

            #[test]
            fn test_missing_as_keyword() {
                let input = InputSpan::new_extra("use foo.bar baz\n", Config::default());
                let result = parse(input);
                // This should fail because 'baz' is not a valid continuation after a use declaration
                // The parser correctly parses "use foo.bar" but then expects a newline
                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 12);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Decl(DeclKind::Use(UseKind::MissingEndOfLine)),
                                cause,
                            } => {
                                // The cause should be the span of "foo.bar"
                                assert_eq!(cause, AstSpan::new(4, 7, 1));
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Expected error for invalid continuation after use declaration"),
                }
            }

            #[test]
            fn test_missing_alias() {
                let input = InputSpan::new_extra("use foo.bar as\n", Config::default());
                let result = parse(input);
                let expected_as_span = AstSpan::new(12, 2, 0);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 14);

                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Decl(DeclKind::AsMissingAlias),
                                cause,
                            } => {
                                assert_eq!(cause, expected_as_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {result:?}"),
                }
            }

            #[test]
            fn test_invalid_path_identifier() {
                // TODO: Add context to this error (in error module): "invalid path identifier"
                let input = InputSpan::new_extra("use 123.bar as baz\n", Config::default());
                let result = parse(input);
                let expected_use_span = AstSpan::new(0, 3, 1);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 4);

                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Decl(DeclKind::Use(UseKind::MissingModelInfo)),
                                cause,
                            } => {
                                assert_eq!(cause, expected_use_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {result:?}"),
                }
            }

            #[test]
            fn test_invalid_alias_identifier() {
                // TODO: Add context to this error (in error module): "invalid alias identifier"
                let input = InputSpan::new_extra("use foo.bar as 123\n", Config::default());
                let result = parse(input);
                let expected_as_span = AstSpan::new(12, 2, 1);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 15);

                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Decl(DeclKind::AsMissingAlias),
                                cause,
                            } => {
                                assert_eq!(cause, expected_as_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {result:?}"),
                }
            }
        }

        mod model_info_error_tests {
            use crate::error::reason::{DeclKind, IncompleteKind, ParserErrorReason};
            use crate::token::error::{ExpectKind, TokenErrorKind};

            use super::*;

            #[test]
            fn test_empty_path() {
                let input = InputSpan::new_extra("", Config::default());
                let result = model_info(input);
                match result {
                    Err(nom::Err::Error(error)) => {
                        assert_eq!(error.error_offset, 0);
                        assert!(matches!(
                            error.reason,
                            ParserErrorReason::TokenError(TokenErrorKind::Expect(
                                ExpectKind::Identifier
                            ))
                        ));
                    }
                    _ => panic!("Expected error for empty path"),
                }
            }

            #[test]
            fn test_invalid_first_identifier() {
                let input = InputSpan::new_extra("123.bar", Config::default());
                let result = model_info(input);
                match result {
                    Err(nom::Err::Error(error)) => {
                        assert_eq!(error.error_offset, 0);
                        assert!(matches!(
                            error.reason,
                            ParserErrorReason::TokenError(TokenErrorKind::Expect(
                                ExpectKind::Identifier
                            ))
                        ));
                    }
                    _ => panic!("Expected error for invalid first identifier"),
                }
            }

            #[test]
            fn test_missing_subcomponent_after_dot() {
                let input = InputSpan::new_extra("foo.", Config::default());
                let result = model_info(input);
                let expected_dot_span = AstSpan::new(3, 1, 0);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 4);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Decl(DeclKind::ModelMissingSubcomponent),
                                cause,
                            } => {
                                assert_eq!(cause, expected_dot_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {result:?}"),
                }
            }

            #[test]
            fn test_invalid_subcomponent_after_dot() {
                let input = InputSpan::new_extra("foo.123", Config::default());
                let result = model_info(input);
                let expected_dot_span = AstSpan::new(3, 1, 0);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 4);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Decl(DeclKind::ModelMissingSubcomponent),
                                cause,
                            } => {
                                assert_eq!(cause, expected_dot_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {result:?}"),
                }
            }

            #[test]
            fn test_dot_at_end() {
                let input = InputSpan::new_extra("foo.bar.", Config::default());
                let result = model_info(input);
                let expected_dot_span = AstSpan::new(7, 1, 0);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 8);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Decl(DeclKind::ModelMissingSubcomponent),
                                cause,
                            } => {
                                assert_eq!(cause, expected_dot_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {result:?}"),
                }
            }
        }

        mod general_error_tests {
            use crate::error::reason::{ExpectKind, IncompleteKind, ParserErrorReason};

            use super::*;

            #[test]
            fn test_no_valid_declaration() {
                let input = InputSpan::new_extra("invalid syntax\n", Config::default());
                let result = parse(input);
                match result {
                    Err(nom::Err::Error(error)) => {
                        assert_eq!(error.error_offset, 0);
                        assert!(matches!(
                            error.reason,
                            ParserErrorReason::Expect(ExpectKind::Decl)
                        ));
                    }
                    _ => panic!("Expected error for no valid declaration"),
                }
            }

            #[test]
            fn test_partial_keyword() {
                let input = InputSpan::new_extra("impor\n", Config::default());
                let result = parse(input);
                match result {
                    Err(nom::Err::Error(error)) => {
                        assert_eq!(error.error_offset, 0);
                        assert!(matches!(
                            error.reason,
                            ParserErrorReason::Expect(ExpectKind::Decl)
                        ));
                    }
                    _ => panic!("Expected error for partial keyword"),
                }
            }

            #[test]
            fn test_wrong_keyword() {
                let input = InputSpan::new_extra("export foo\n", Config::default());
                let result = parse(input);
                match result {
                    Err(nom::Err::Error(error)) => {
                        assert_eq!(error.error_offset, 0);
                        assert!(matches!(
                            error.reason,
                            ParserErrorReason::Expect(ExpectKind::Decl)
                        ));
                    }
                    _ => panic!("Expected error for wrong keyword"),
                }
            }

            #[test]
            fn test_mixed_case_keywords() {
                let input = InputSpan::new_extra("Import foo\n", Config::default());
                let result = parse(input);
                match result {
                    Err(nom::Err::Error(error)) => {
                        assert_eq!(error.error_offset, 0);
                        assert!(matches!(
                            error.reason,
                            ParserErrorReason::Expect(ExpectKind::Decl)
                        ));
                    }
                    _ => panic!("Expected error for mixed case keywords"),
                }
            }

            #[test]
            fn test_symbols_only() {
                let input = InputSpan::new_extra("+++---\n", Config::default());
                let result = parse(input);
                match result {
                    Err(nom::Err::Error(error)) => {
                        assert_eq!(error.error_offset, 0);
                        assert!(matches!(
                            error.reason,
                            ParserErrorReason::Expect(ExpectKind::Decl)
                        ));
                    }
                    _ => panic!("Expected error for symbols only"),
                }
            }

            #[test]
            fn test_numbers_only() {
                let input = InputSpan::new_extra("123 456\n", Config::default());
                let result = parse(input);
                match result {
                    Err(nom::Err::Error(error)) => {
                        assert_eq!(error.error_offset, 0);
                        assert!(matches!(
                            error.reason,
                            ParserErrorReason::Expect(ExpectKind::Decl)
                        ));
                    }
                    _ => panic!("Expected error for numbers only"),
                }
            }

            #[test]
            fn test_parse_complete_with_remaining_input() {
                let input = InputSpan::new_extra("import foo\nrest", Config::default());
                let result = parse_complete(input);
                match result {
                    Err(nom::Err::Error(error)) => {
                        assert_eq!(error.error_offset, 11);
                        assert!(matches!(error.reason, ParserErrorReason::UnexpectedToken));
                    }
                    _ => panic!("Expected error for parse complete with remaining input"),
                }
            }

            #[test]
            fn test_use_decl_with_unclosed_bracket_empty() {
                let input = InputSpan::new_extra("use foo with [\n", Config::default());
                let result = parse(input);
                let expected_bracket_span = AstSpan::new(13, 1, 0);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 15);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::UnclosedBracket,
                                cause,
                            } => {
                                assert_eq!(cause, expected_bracket_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {result:?}"),
                }
            }

            #[test]
            fn test_use_decl_with_unclosed_bracket_single_submodel() {
                let input = InputSpan::new_extra("use foo with [bar\n", Config::default());
                let result = parse(input);
                let expected_bracket_span = AstSpan::new(13, 1, 0);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 18);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::UnclosedBracket,
                                cause,
                            } => {
                                assert_eq!(cause, expected_bracket_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {result:?}"),
                }
            }

            #[test]
            fn test_use_decl_with_unclosed_bracket_multiple_submodels() {
                let input = InputSpan::new_extra("use foo with [bar, baz\n", Config::default());
                let result = parse(input);
                let expected_bracket_span = AstSpan::new(13, 1, 0);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 23);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::UnclosedBracket,
                                cause,
                            } => {
                                assert_eq!(cause, expected_bracket_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {result:?}"),
                }
            }

            #[test]
            fn test_use_decl_with_unclosed_bracket_with_trailing_comma() {
                let input = InputSpan::new_extra("use foo with [bar, baz,\n", Config::default());
                let result = parse(input);
                let expected_bracket_span = AstSpan::new(13, 1, 0);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 24);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::UnclosedBracket,
                                cause,
                            } => {
                                assert_eq!(cause, expected_bracket_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {result:?}"),
                }
            }

            #[test]
            fn test_use_decl_with_unclosed_bracket_with_subcomponents() {
                let input = InputSpan::new_extra("use foo with [bar.qux, baz.quux\n", Config::default());
                let result = parse(input);
                let expected_bracket_span = AstSpan::new(13, 1, 0);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 32);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::UnclosedBracket,
                                cause,
                            } => {
                                assert_eq!(cause, expected_bracket_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {result:?}"),
                }
            }

            #[test]
            fn test_use_decl_with_unclosed_bracket_with_aliases() {
                let input =
                    InputSpan::new_extra("use foo with [bar as baz, qux as quux\n", Config::default());
                let result = parse(input);
                let expected_bracket_span = AstSpan::new(13, 1, 0);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 38);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::UnclosedBracket,
                                cause,
                            } => {
                                assert_eq!(cause, expected_bracket_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {result:?}"),
                }
            }

            #[test]
            fn test_use_decl_with_unclosed_bracket_with_model_alias() {
                let input = InputSpan::new_extra("use foo as bar with [qux, baz\n", Config::default());
                let result = parse(input);
                let expected_bracket_span = AstSpan::new(20, 1, 0);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 30);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::UnclosedBracket,
                                cause,
                            } => {
                                assert_eq!(cause, expected_bracket_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {result:?}"),
                }
            }

            #[test]
            fn test_use_decl_with_unclosed_bracket_with_newlines() {
                let input = InputSpan::new_extra("use foo with [\nbar,\nbaz\n", Config::default());
                let result = parse(input);
                let expected_bracket_span = AstSpan::new(13, 1, 0);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 24);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::UnclosedBracket,
                                cause,
                            } => {
                                assert_eq!(cause, expected_bracket_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {result:?}"),
                }
            }

            #[test]
            fn test_use_decl_with_unclosed_bracket_with_complex_path() {
                let input = InputSpan::new_extra(
                    "use utils/math.trigonometry as trig with [sin, cos as cosine\n",
                    Config::default(),
                );
                let result = parse(input);
                let expected_bracket_span = AstSpan::new(41, 1, 0);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 61);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::UnclosedBracket,
                                cause,
                            } => {
                                assert_eq!(cause, expected_bracket_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {result:?}"),
                }
            }
        }
    }
}
