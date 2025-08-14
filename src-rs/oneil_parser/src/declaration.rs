//! Parser for declarations in an Oneil program.

use nom::{Parser as _, branch::alt, combinator::all_consuming, multi::many0};

use oneil_ast::{
    Span as AstSpan,
    declaration::{Decl, DeclNode, Import, UseModel},
    naming::{Identifier, IdentifierNode},
    node::Node,
};

use crate::{
    error::{ErrorHandlingParser, ParserError},
    parameter::parse as parse_parameter,
    test::parse as parse_test,
    token::{
        keyword::{as_, from, import, use_},
        naming::identifier,
        structure::end_of_line,
        symbol::dot,
    },
    util::{Result, Span},
};

/// Parses a declaration
///
/// This function **may not consume the complete input**.
pub fn parse(input: Span) -> Result<DeclNode, ParserError> {
    decl.parse(input)
}

/// Parses a declaration
///
/// This function **fails if the complete input is not consumed**.
pub fn parse_complete(input: Span) -> Result<DeclNode, ParserError> {
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
fn decl(input: Span) -> Result<DeclNode, ParserError> {
    alt((import_decl, from_decl, use_decl, parameter_decl, test_decl))
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
fn import_decl(input: Span) -> Result<DeclNode, ParserError> {
    let (rest, import_token) = import.convert_errors().parse(input)?;

    let (rest, import_path_token) = identifier
        .or_fail_with(ParserError::import_missing_path(&import_token))
        .parse(rest)?;

    let (rest, end_of_line_token) = end_of_line
        .or_fail_with(ParserError::import_missing_end_of_line(&import_path_token))
        .parse(rest)?;

    let span =
        AstSpan::calc_span_with_whitespace(&import_token, &import_path_token, &end_of_line_token);

    let import_path = Node::new(import_path_token, import_path_token.lexeme().to_string());
    let import_node = Node::new(span, Import::new(import_path));

    let decl_node = Node::new(span, Decl::Import(import_node));

    Ok((rest, decl_node))
}

/// Parses a from declaration
///
/// A from declaration has the format: `from path use model [inputs] as alias` followed by a newline.
/// This declaration imports a specific model from a module and gives it a local alias.
///
/// Examples:
/// - `from foo.bar use model as baz`
/// - `from utils.math use model(x=1, y=2) as calculator`
/// - `from my_module.submodule use model as local_name`
///
/// The parser requires:
/// - The `from` keyword
/// - A model path (e.g., "foo.bar")
/// - The `use` keyword
/// - The `model` keyword
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
/// Returns a declaration node containing the parsed from declaration, or an error if
/// the declaration is malformed.
fn from_decl(input: Span) -> Result<DeclNode, ParserError> {
    let (rest, from_token) = from.convert_errors().parse(input)?;

    let (rest, (from_path, mut subcomponents)) = model_path
        .or_fail_with(ParserError::from_missing_path(&from_token))
        .parse(rest)?;

    // for error reporting
    let from_path_span = match subcomponents.last() {
        Some(last) => AstSpan::calc_span(&from_path, last),
        None => AstSpan::from(&from_path),
    };

    let (rest, use_token) = use_
        .or_fail_with(ParserError::from_missing_use(&from_path_span))
        .parse(rest)?;

    let (rest, use_model) = identifier
        .or_fail_with(ParserError::from_missing_use_model(&use_token))
        .parse(rest)?;
    let use_model = Node::new(use_model, Identifier::new(use_model.lexeme().to_string()));
    let use_model_span = AstSpan::from(&use_model);
    subcomponents.push(use_model);

    let (rest, as_token) = as_
        .or_fail_with(ParserError::from_missing_as(&use_model_span))
        .parse(rest)?;

    let (rest, alias) = identifier
        .or_fail_with(ParserError::from_missing_alias(&as_token))
        .parse(rest)?;
    let alias = Node::new(alias, Identifier::new(alias.lexeme().to_string()));

    let (rest, end_of_line_token) = end_of_line
        .or_fail_with(ParserError::from_missing_end_of_line(&alias))
        .parse(rest)?;

    let span = AstSpan::calc_span_with_whitespace(&from_token, &alias, &end_of_line_token);

    let use_model_node = Node::new(span, UseModel::new(from_path, subcomponents, Some(alias)));

    let decl_node = Node::new(span, Decl::UseModel(use_model_node));

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
fn use_decl(input: Span) -> Result<DeclNode, ParserError> {
    let (rest, use_token) = use_.convert_errors().parse(input)?;

    let (rest, (path, subcomponents)) = model_path
        .or_fail_with(ParserError::use_missing_path(&use_token))
        .parse(rest)?;

    // for error reporting
    let use_path_span = match subcomponents.last() {
        Some(last) => AstSpan::calc_span(&path, last),
        None => AstSpan::from(&path),
    };

    let (rest, as_token) = as_
        .or_fail_with(ParserError::use_missing_as(&use_path_span))
        .parse(rest)?;

    let (rest, alias) = identifier
        .or_fail_with(ParserError::use_missing_alias(&as_token))
        .parse(rest)?;
    let alias = Node::new(alias, Identifier::new(alias.lexeme().to_string()));

    let (rest, end_of_line_token) = end_of_line
        .or_fail_with(ParserError::use_missing_end_of_line(&alias))
        .parse(rest)?;

    let span = AstSpan::calc_span_with_whitespace(&use_token, &alias, &end_of_line_token);

    let use_model_node = Node::new(span, UseModel::new(path, subcomponents, Some(alias)));

    let decl_node = Node::new(span, Decl::UseModel(use_model_node));

    Ok((rest, decl_node))
}

/// Parses a model path (e.g., "foo.bar.baz")
///
/// A model path consists of a sequence of identifiers separated by dots.
/// The first identifier is the main module name, and subsequent identifiers
/// are subcomponents or nested modules.
///
/// Examples:
/// - `foo` (single component)
/// - `foo.bar` (two components)
/// - `foo.bar.baz` (three components)
/// - `utils.math.functions` (multiple components)
///
/// The parser returns:
/// - The first identifier as the main path
/// - A vector of subsequent identifiers as subcomponents
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns a tuple containing the main path identifier and a vector of
/// subcomponent identifiers, or an error if the path is malformed.
fn model_path(input: Span) -> Result<(IdentifierNode, Vec<IdentifierNode>), ParserError> {
    let (rest, path) = identifier.convert_errors().parse(input)?;
    let path = Node::new(path, Identifier::new(path.lexeme().to_string()));

    let (rest, subcomponents) = many0(|input| {
        let (rest, dot_token) = dot.convert_errors().parse(input)?;
        let (rest, subcomponent) = identifier
            .or_fail_with(ParserError::model_path_missing_subcomponent(&dot_token))
            .parse(rest)?;
        let subcomponent_node = Node::new(
            subcomponent,
            Identifier::new(subcomponent.lexeme().to_string()),
        );
        Ok((rest, subcomponent_node))
    })
    .parse(rest)?;

    Ok((rest, (path, subcomponents)))
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
fn parameter_decl(input: Span) -> Result<DeclNode, ParserError> {
    let (rest, parameter) = parse_parameter.parse(input)?;

    let span = AstSpan::from(&parameter);
    let decl_node = Node::new(span, Decl::Parameter(parameter));

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
fn test_decl(input: Span) -> Result<DeclNode, ParserError> {
    let (rest, test) = parse_test.parse(input)?;

    let span = AstSpan::from(&test);
    let decl_node = Node::new(span, Decl::Test(test));

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
            let (rest, decl) = parse(input).unwrap();
            match decl.node_value() {
                Decl::Import(import_node) => {
                    let import_path = import_node.path();

                    assert_eq!(import_path.node_value(), "foo");
                    assert_eq!(import_path.node_span(), &AstSpan::new(7, 3, 0));
                }
                _ => panic!("Expected import declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_decl() {
            let input = Span::new_extra("use foo.bar as baz\n", Config::default());
            let (rest, decl) = parse(input).unwrap();
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model = use_model_node.node_value();
                    assert_eq!(use_model.model_name().as_str(), "foo");
                    assert_eq!(use_model.subcomponents()[0].as_str(), "bar");
                    assert_eq!(use_model.alias().unwrap().as_str(), "baz");
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_from_decl() {
            let input = Span::new_extra("from foo.bar use model as baz\n", Config::default());
            let (rest, decl) = parse(input).unwrap();
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model = use_model_node.node_value();
                    assert_eq!(use_model.model_name().as_str(), "foo");
                    assert_eq!(use_model.subcomponents()[0].as_str(), "bar");
                    assert_eq!(use_model.alias().unwrap().as_str(), "baz");
                }
                _ => panic!("Expected from declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_parse_complete_import_success() {
            let input = Span::new_extra("import foo\n", Config::default());
            let (rest, decl) = parse_complete(input).unwrap();
            match decl.node_value() {
                Decl::Import(import_node) => {
                    let import_path = import_node.path();
                    assert_eq!(import_path.node_value(), "foo");
                    assert_eq!(import_path.node_span(), &AstSpan::new(7, 3, 0));
                }
                _ => panic!("Expected import declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_parse_complete_use_success() {
            let input = Span::new_extra("use foo.bar as baz\n", Config::default());
            let (rest, decl) = parse_complete(input).unwrap();
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model = use_model_node.node_value();
                    assert_eq!(use_model.model_name().as_str(), "foo");
                    assert_eq!(use_model.subcomponents()[0].as_str(), "bar");
                    assert_eq!(use_model.alias().unwrap().as_str(), "baz");
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_parse_complete_from_success() {
            let input = Span::new_extra("from foo.bar use model as baz\n", Config::default());
            let (rest, decl) = parse_complete(input).unwrap();
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model = use_model_node.node_value();
                    assert_eq!(use_model.model_name().as_str(), "foo");
                    assert_eq!(use_model.subcomponents()[0].as_str(), "bar");
                    assert_eq!(use_model.alias().unwrap().as_str(), "baz");
                }
                _ => panic!("Expected from declaration"),
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
                    _ => panic!("Unexpected result {:?}", result),
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
                    _ => panic!("Unexpected result {:?}", result),
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
                    _ => panic!("Unexpected result {:?}", result),
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
                let expected_foo_bar_span = AstSpan::new(4, 7, 1);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 12);

                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Decl(DeclKind::Use(UseKind::MissingAs)),
                                cause,
                            } => {
                                assert_eq!(cause, expected_foo_bar_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
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
                                kind: IncompleteKind::Decl(DeclKind::Use(UseKind::MissingAlias)),
                                cause,
                            } => {
                                assert_eq!(cause, expected_as_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
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
                    _ => panic!("Unexpected result {:?}", result),
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
                                kind: IncompleteKind::Decl(DeclKind::Use(UseKind::MissingAlias)),
                                cause,
                            } => {
                                assert_eq!(cause, expected_as_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }
        }

        mod from_error_tests {
            use crate::error::reason::{
                DeclKind, ExpectKind, FromKind, IncompleteKind, ParserErrorReason,
            };

            use super::*;

            #[test]
            fn test_missing_from_keyword() {
                let input = Span::new_extra("foo.bar use model as baz\n", Config::default());
                let result = parse(input);
                match result {
                    Err(nom::Err::Error(error)) => {
                        assert_eq!(error.error_offset, 0);
                        assert!(matches!(
                            error.reason,
                            ParserErrorReason::Expect(ExpectKind::Decl)
                        ));
                    }
                    _ => panic!("Expected error for missing from keyword"),
                }
            }

            #[test]
            fn test_missing_use_keyword() {
                let input = Span::new_extra("from foo.bar model as baz\n", Config::default());
                let result = parse(input);
                let expected_foo_bar_span = AstSpan::new(5, 7, 1);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 13);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Decl(DeclKind::From(FromKind::MissingUse)),
                                cause,
                            } => {
                                assert_eq!(cause, expected_foo_bar_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_missing_model_keyword() {
                let input = Span::new_extra("from foo.bar use as baz\n", Config::default());
                let result = parse(input);
                let expected_use_span = AstSpan::new(13, 3, 1);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 17);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind:
                                    IncompleteKind::Decl(DeclKind::From(FromKind::MissingUseModel)),
                                cause,
                            } => {
                                assert_eq!(cause, expected_use_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_missing_as_keyword() {
                let input = Span::new_extra("from foo.bar use model baz\n", Config::default());
                let result = parse(input);
                let expected_model_span = AstSpan::new(17, 5, 1);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 23);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Decl(DeclKind::From(FromKind::MissingAs)),
                                cause,
                            } => {
                                assert_eq!(cause, expected_model_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_missing_alias() {
                let input = Span::new_extra("from foo.bar use model as\n", Config::default());
                let result = parse(input);
                let expected_as_span = AstSpan::new(23, 2, 0);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 25);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Decl(DeclKind::From(FromKind::MissingAlias)),
                                cause,
                            } => {
                                assert_eq!(cause, expected_as_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_invalid_path_identifier() {
                let input = Span::new_extra("from 123.bar use model as baz\n", Config::default());
                let result = parse(input);
                let expected_from_span = AstSpan::new(0, 4, 1);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 5);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Decl(DeclKind::From(FromKind::MissingPath)),
                                cause,
                            } => {
                                assert_eq!(cause, expected_from_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_invalid_model_identifier() {
                let input = Span::new_extra("from foo.bar use 123 as baz\n", Config::default());
                let result = parse(input);
                let expected_use_span = AstSpan::new(13, 3, 1);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 17);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind:
                                    IncompleteKind::Decl(DeclKind::From(FromKind::MissingUseModel)),
                                cause,
                            } => {
                                assert_eq!(cause, expected_use_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_invalid_alias_identifier() {
                let input = Span::new_extra("from foo.bar use model as 123\n", Config::default());
                let result = parse(input);
                let expected_as_span = AstSpan::new(23, 2, 1);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 26);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Decl(DeclKind::From(FromKind::MissingAlias)),
                                cause,
                            } => {
                                assert_eq!(cause, expected_as_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
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
                    _ => panic!("Unexpected result {:?}", result),
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
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_consecutive_dots() {
                let input = Span::new_extra("foo..bar", Config::default());
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
                    _ => panic!("Unexpected result {:?}", result),
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
                    _ => panic!("Unexpected result {:?}", result),
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
