//! Parser for declarations in an Oneil program.

use nom::{
    Parser as _,
    branch::alt,
    combinator::{all_consuming, opt},
    multi::many0,
};

use oneil_ast::{
    Span as AstSpan,
    declaration::{Decl, DeclNode, Import, UseModel},
    naming::{Directory, DirectoryNode, Identifier, IdentifierNode},
    node::Node,
};

use crate::{
    error::{ErrorHandlingParser, ParserError},
    parameter::parse as parse_parameter,
    test::parse as parse_test,
    token::{
        keyword::{as_, import, use_},
        naming::identifier,
        structure::end_of_line,
        symbol::{dot, dot_dot, slash},
    },
    util::{Result, Span},
};

/// Parses a declaration
///
/// This function **may not consume the complete input**.
pub fn parse(input: Span<'_>) -> Result<'_, DeclNode, ParserError> {
    decl.parse(input)
}

/// Parses a declaration
///
/// This function **fails if the complete input is not consumed**.
pub fn parse_complete(input: Span<'_>) -> Result<'_, DeclNode, ParserError> {
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
fn decl(input: Span<'_>) -> Result<'_, DeclNode, ParserError> {
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
fn import_decl(input: Span<'_>) -> Result<'_, DeclNode, ParserError> {
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
fn use_decl(input: Span<'_>) -> Result<'_, DeclNode, ParserError> {
    let (rest, use_token) = use_.convert_errors().parse(input)?;

    let (rest, (directory_path, model_name, subcomponents)) = model_path
        .or_fail_with(ParserError::use_missing_path(&use_token))
        .parse(rest)?;

    // for error reporting
    let use_path_span = match (directory_path.first(), subcomponents.last()) {
        (Some(first), Some(last)) => AstSpan::calc_span(&first, &last),
        (Some(first), None) => AstSpan::calc_span(&first, &model_name),
        (None, Some(last)) => AstSpan::calc_span(&model_name, &last),
        (None, None) => AstSpan::from(&model_name),
    };

    let (rest, alias) = opt(as_alias).parse(rest)?;

    let final_span = match &alias {
        Some(alias) => AstSpan::from(alias),
        None => use_path_span,
    };

    let (rest, end_of_line_token) = end_of_line
        .or_fail_with(ParserError::use_missing_end_of_line(&final_span))
        .parse(rest)?;

    let span = AstSpan::calc_span_with_whitespace(&use_token, &final_span, &end_of_line_token);

    let use_model_node = Node::new(
        &span,
        UseModel::new(model_name, subcomponents, directory_path, alias),
    );

    let decl_node = Node::new(&span, Decl::UseModel(use_model_node));

    Ok((rest, decl_node))
}

/// Parses a model path (e.g., "foo/bar.baz")
///
/// A model path consists of an optional directory path followed by a model name and optional subcomponents.
/// The directory path is a sequence of directory names separated by forward slashes.
/// The model name is a single identifier.
/// The subcomponents are a sequence of identifiers separated by dots.
///
/// Examples:
/// - `foo` (just model name)
/// - `foo.bar` (model name with subcomponent)
/// - `foo.bar.baz` (model name with multiple subcomponents)
/// - `path/to/foo` (directory path with model name)
/// - `path/to/foo.bar` (directory path, model name, and subcomponent)
/// - `../foo` (parent directory with model name)
/// - `./foo` (current directory with model name)
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns a tuple containing:
/// - A vector of directory nodes representing the directory path (empty if no path)
/// - A node containing the model name identifier
/// - A vector of identifier nodes for any subcomponents (empty if none)
///
/// Returns an error if the path is malformed.
fn model_path(
    input: Span<'_>,
) -> Result<'_, (Vec<DirectoryNode>, IdentifierNode, Vec<IdentifierNode>), ParserError> {
    let (rest, directory_path) = directory_path(input)?;

    let (rest, model_name_token) = identifier.convert_errors().parse(rest)?;
    let model_name = Node::new(
        &model_name_token,
        Identifier::new(model_name_token.lexeme().to_string()),
    );

    let (rest, subcomponents) = submodel_components(rest)?;

    Ok((rest, (directory_path, model_name, subcomponents)))
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
fn directory_path(input: Span<'_>) -> Result<'_, Vec<DirectoryNode>, ParserError> {
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
fn directory_name(input: Span<'_>) -> Result<'_, DirectoryNode, ParserError> {
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

/// Parses submodel components in a model path
///
/// Submodel components are a sequence of identifiers separated by dots.
/// Each identifier represents a subcomponent of the model.
///
/// Examples:
/// - `.foo` (single subcomponent)
/// - `.foo.bar` (multiple subcomponents)
/// - ` ` (no subcomponents)
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns a vector of identifier nodes representing the subcomponents.
/// Returns an error if a dot is not followed by a valid identifier.
fn submodel_components(input: Span<'_>) -> Result<'_, Vec<IdentifierNode>, ParserError> {
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
fn as_alias(input: Span<'_>) -> Result<'_, IdentifierNode, ParserError> {
    let (rest, as_token) = as_.convert_errors().parse(input)?;

    let (rest, alias) = identifier
        .or_fail_with(ParserError::as_missing_alias(&as_token))
        .parse(rest)?;
    let alias_node = Node::new(&alias, Identifier::new(alias.lexeme().to_string()));

    Ok((rest, alias_node))
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
fn parameter_decl(input: Span<'_>) -> Result<'_, DeclNode, ParserError> {
    let (rest, parameter) = parse_parameter.parse(input)?;

    let span = AstSpan::from(&parameter);
    let decl_node = Node::new(&span, Decl::Parameter(Box::new(parameter)));

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
fn test_decl(input: Span<'_>) -> Result<'_, DeclNode, ParserError> {
    let (rest, test) = parse_test.parse(input)?;

    let span = AstSpan::from(&test);
    let decl_node = Node::new(&span, Decl::Test(Box::new(test)));

    Ok((rest, decl_node))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;

    mod success_tests {
        use super::*;

        #[test]
        fn test_import_decl() {
            let input = Span::new_extra("import foo\n", Config::default());
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
        fn test_use_decl() {
            let input = Span::new_extra("use foo.bar as baz\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model = use_model_node.node_value();
                    let alias = use_model.alias().expect("alias should be present");
                    assert_eq!(use_model.model_name().as_str(), "foo");
                    assert_eq!(use_model.subcomponents()[0].as_str(), "bar");
                    assert_eq!(alias.as_str(), "baz");
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_decl_without_alias() {
            let input = Span::new_extra("use foo.bar\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model = use_model_node.node_value();
                    assert_eq!(use_model.model_name().as_str(), "foo");
                    assert_eq!(use_model.subcomponents()[0].as_str(), "bar");
                    assert!(use_model.alias().is_none());
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_decl_simple_without_alias() {
            let input = Span::new_extra("use foo\n", Config::default());
            let (rest, decl) = parse(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model = use_model_node.node_value();
                    assert_eq!(use_model.model_name().as_str(), "foo");
                    assert_eq!(use_model.subcomponents().len(), 0);
                    assert!(use_model.alias().is_none());
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_parse_complete_import_success() {
            let input = Span::new_extra("import foo\n", Config::default());
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
            let input = Span::new_extra("use foo.bar as baz\n", Config::default());
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model = use_model_node.node_value();
                    let alias = use_model.alias().expect("alias should be present");
                    assert_eq!(use_model.model_name().as_str(), "foo");
                    assert_eq!(use_model.subcomponents()[0].as_str(), "bar");
                    assert_eq!(alias.as_str(), "baz");
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_with_single_directory() {
            let input = Span::new_extra("use utils/math as calculator\n", Config::default());
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model = use_model_node.node_value();
                    let alias = use_model.alias().expect("alias should be present");
                    assert_eq!(use_model.model_name().as_str(), "math");
                    assert_eq!(use_model.subcomponents().len(), 0);
                    assert_eq!(alias.as_str(), "calculator");

                    // Check directory path
                    assert_eq!(use_model.directory_path().len(), 1);
                    assert_eq!(use_model.directory_path()[0].node_value().as_str(), "utils");
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_with_single_directory_without_alias() {
            let input = Span::new_extra("use utils/math\n", Config::default());
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model = use_model_node.node_value();
                    assert_eq!(use_model.model_name().as_str(), "math");
                    assert_eq!(use_model.subcomponents().len(), 0);
                    assert!(use_model.alias().is_none());

                    // Check directory path
                    assert_eq!(use_model.directory_path().len(), 1);
                    assert_eq!(use_model.directory_path()[0].node_value().as_str(), "utils");
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_with_multiple_directories() {
            let input = Span::new_extra(
                "use models/physics/mechanics as dynamics\n",
                Config::default(),
            );
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model = use_model_node.node_value();
                    let alias = use_model.alias().expect("alias should be present");
                    assert_eq!(use_model.model_name().as_str(), "mechanics");
                    assert_eq!(use_model.subcomponents().len(), 0);
                    assert_eq!(alias.as_str(), "dynamics");

                    // Check directory path
                    assert_eq!(use_model.directory_path().len(), 2);
                    assert_eq!(
                        use_model.directory_path()[0].node_value().as_str(),
                        "models"
                    );
                    assert_eq!(
                        use_model.directory_path()[1].node_value().as_str(),
                        "physics"
                    );
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_with_directory_and_subcomponents() {
            let input = Span::new_extra("use utils/math.trigonometry as trig\n", Config::default());
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model = use_model_node.node_value();
                    let alias = use_model.alias().expect("alias should be present");
                    assert_eq!(use_model.model_name().as_str(), "math");
                    assert_eq!(use_model.subcomponents().len(), 1);
                    assert_eq!(use_model.subcomponents()[0].as_str(), "trigonometry");
                    assert_eq!(alias.as_str(), "trig");

                    // Check directory path
                    assert_eq!(use_model.directory_path().len(), 1);
                    assert_eq!(use_model.directory_path()[0].node_value().as_str(), "utils");
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_with_current_directory() {
            let input = Span::new_extra("use ./local_model as local\n", Config::default());
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model = use_model_node.node_value();
                    let alias = use_model.alias().expect("alias should be present");
                    assert_eq!(use_model.model_name().as_str(), "local_model");
                    assert_eq!(use_model.subcomponents().len(), 0);
                    assert_eq!(alias.as_str(), "local");

                    // Check directory path
                    assert_eq!(use_model.directory_path().len(), 1);
                    assert_eq!(use_model.directory_path()[0].node_value().as_str(), ".");
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_with_parent_directory() {
            let input = Span::new_extra("use ../parent_model as parent\n", Config::default());
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model = use_model_node.node_value();
                    let alias = use_model.alias().expect("alias should be present");
                    assert_eq!(use_model.model_name().as_str(), "parent_model");
                    assert_eq!(use_model.subcomponents().len(), 0);
                    assert_eq!(alias.as_str(), "parent");

                    // Check directory path
                    assert_eq!(use_model.directory_path().len(), 1);
                    assert_eq!(use_model.directory_path()[0].node_value().as_str(), "..");
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_with_mixed_directory_types() {
            let input = Span::new_extra(
                "use ../shared/./utils/math as shared_math\n",
                Config::default(),
            );
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model = use_model_node.node_value();
                    let alias = use_model.alias().expect("alias should be present");
                    assert_eq!(use_model.model_name().as_str(), "math");
                    assert_eq!(use_model.subcomponents().len(), 0);
                    assert_eq!(alias.as_str(), "shared_math");

                    // Check directory path
                    assert_eq!(use_model.directory_path().len(), 4);
                    assert_eq!(use_model.directory_path()[0].node_value().as_str(), "..");
                    assert_eq!(
                        use_model.directory_path()[1].node_value().as_str(),
                        "shared"
                    );
                    assert_eq!(use_model.directory_path()[2].node_value().as_str(), ".");
                    assert_eq!(use_model.directory_path()[3].node_value().as_str(), "utils");
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_with_complex_path_and_subcomponents() {
            let input = Span::new_extra(
                "use models/physics/mechanics.rotational.dynamics as rotation\n",
                Config::default(),
            );
            let (rest, decl) = parse_complete(input).expect("parsing should succeed");
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model = use_model_node.node_value();
                    let alias = use_model.alias().expect("alias should be present");
                    assert_eq!(use_model.model_name().as_str(), "mechanics");
                    assert_eq!(use_model.subcomponents().len(), 2);
                    assert_eq!(use_model.subcomponents()[0].as_str(), "rotational");
                    assert_eq!(use_model.subcomponents()[1].as_str(), "dynamics");
                    assert_eq!(alias.as_str(), "rotation");

                    // Check directory path
                    assert_eq!(use_model.directory_path().len(), 2);
                    assert_eq!(
                        use_model.directory_path()[0].node_value().as_str(),
                        "models"
                    );
                    assert_eq!(
                        use_model.directory_path()[1].node_value().as_str(),
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
            let input = Span::new_extra("..", Config::default());
            let (rest, dir) = directory_name(input).expect("should parse parent directory");
            assert_eq!(dir.node_value().as_str(), "..");
            assert_eq!(rest.fragment(), &"");

            // Test current directory
            let input = Span::new_extra(".", Config::default());
            let (rest, dir) = directory_name(input).expect("should parse current directory");
            assert_eq!(dir.node_value().as_str(), ".");
            assert_eq!(rest.fragment(), &"");

            // Test regular directory name
            let input = Span::new_extra("foo", Config::default());
            let (rest, dir) = directory_name(input).expect("should parse regular directory name");
            assert_eq!(dir.node_value().as_str(), "foo");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_mixed_directory_path_parsing() {
            let input = Span::new_extra("../shared/./utils/math", Config::default());
            let (_rest, (directory_path, model_name, subcomponents)) =
                model_path(input).expect("should parse mixed directory path");

            println!("Directory path length: {}", directory_path.len());
            for (i, dir) in directory_path.iter().enumerate() {
                println!("Directory {}: {}", i, dir.node_value().as_str());
            }
            println!("Model name: {}", model_name.node_value().as_str());
            println!(
                "Subcomponents: {:?}",
                subcomponents
                    .iter()
                    .map(|s| s.node_value().as_str())
                    .collect::<Vec<_>>()
            );

            assert_eq!(directory_path.len(), 4);
            assert_eq!(directory_path[0].node_value().as_str(), "..");
            assert_eq!(directory_path[1].node_value().as_str(), "shared");
            assert_eq!(directory_path[2].node_value().as_str(), ".");
            assert_eq!(directory_path[3].node_value().as_str(), "utils");
            assert_eq!(model_name.node_value().as_str(), "math");
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
                let input = Span::new_extra("", Config::default());
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
                let input = Span::new_extra("foo\n", Config::default());
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
                let input = Span::new_extra("import\n", Config::default());
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
                let input = Span::new_extra("import 123\n", Config::default());
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
                let input = Span::new_extra("import foo@bar\n", Config::default());
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
                let input = Span::new_extra("   \n", Config::default());
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
                let input = Span::new_extra("# comment\n", Config::default());
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
                let input = Span::new_extra("foo.bar as baz\n", Config::default());
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
                let input = Span::new_extra("use foo.bar baz\n", Config::default());
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
                let input = Span::new_extra("use foo.bar as\n", Config::default());
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
                let input = Span::new_extra("use 123.bar as baz\n", Config::default());
                let result = parse(input);
                let expected_use_span = AstSpan::new(0, 3, 1);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 4);

                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Decl(DeclKind::Use(UseKind::MissingPath)),
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
                let input = Span::new_extra("use foo.bar as 123\n", Config::default());
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

        mod model_path_error_tests {
            use crate::error::reason::{DeclKind, IncompleteKind, ParserErrorReason};
            use crate::token::error::{ExpectKind, TokenErrorKind};

            use super::*;

            #[test]
            fn test_empty_path() {
                let input = Span::new_extra("", Config::default());
                let result = model_path(input);
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
                let input = Span::new_extra("123.bar", Config::default());
                let result = model_path(input);
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
                let input = Span::new_extra("foo.", Config::default());
                let result = model_path(input);
                let expected_dot_span = AstSpan::new(3, 1, 0);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 4);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Decl(DeclKind::ModelPathMissingSubcomponent),
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
                let input = Span::new_extra("foo.123", Config::default());
                let result = model_path(input);
                let expected_dot_span = AstSpan::new(3, 1, 0);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 4);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Decl(DeclKind::ModelPathMissingSubcomponent),
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
                let input = Span::new_extra("foo.bar.", Config::default());
                let result = model_path(input);
                let expected_dot_span = AstSpan::new(7, 1, 0);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 8);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Decl(DeclKind::ModelPathMissingSubcomponent),
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
            use crate::error::reason::{ExpectKind, ParserErrorReason};

            use super::*;

            #[test]
            fn test_no_valid_declaration() {
                let input = Span::new_extra("invalid syntax\n", Config::default());
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
                let input = Span::new_extra("impor\n", Config::default());
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
                let input = Span::new_extra("export foo\n", Config::default());
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
                let input = Span::new_extra("Import foo\n", Config::default());
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
                let input = Span::new_extra("+++---\n", Config::default());
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
                let input = Span::new_extra("123 456\n", Config::default());
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
                let input = Span::new_extra("import foo\nrest", Config::default());
                let result = parse_complete(input);
                match result {
                    Err(nom::Err::Error(error)) => {
                        assert_eq!(error.error_offset, 11);
                        assert!(matches!(error.reason, ParserErrorReason::UnexpectedToken));
                    }
                    _ => panic!("Expected error for parse complete with remaining input"),
                }
            }
        }
    }
}
