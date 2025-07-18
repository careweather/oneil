//! Parser for declarations in an Oneil program.

use nom::{
    Parser as _,
    branch::alt,
    combinator::{all_consuming, opt},
    multi::{many0, separated_list0},
};

use oneil_ast::{
    Span as AstSpan,
    declaration::{
        Decl, DeclNode, Import, ModelInput, ModelInputList, ModelInputListNode, ModelInputNode,
        UseModel,
    },
    naming::{Identifier, IdentifierNode},
    node::Node,
};

use crate::{
    error::{ErrorHandlingParser, ParserError},
    expression::parse as parse_expr,
    parameter::parse as parse_parameter,
    test::parse as parse_test,
    token::{
        keyword::{as_, from, import, use_},
        naming::identifier,
        structure::end_of_line,
        symbol::{comma, dot, equals, paren_left, paren_right},
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

    let import_node = Node::new(span, Import::new(import_path_token.lexeme().to_string()));

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

    let (rest, inputs) = opt(model_inputs).parse(rest)?;

    // for error reporting
    let use_model_or_inputs_span = inputs
        .as_ref()
        .map_or(use_model_span, |inputs| AstSpan::from(inputs));

    let (rest, as_token) = as_
        .or_fail_with(ParserError::from_missing_as(&use_model_or_inputs_span))
        .parse(rest)?;

    let (rest, alias) = identifier
        .or_fail_with(ParserError::from_missing_alias(&as_token))
        .parse(rest)?;
    let alias = Node::new(alias, Identifier::new(alias.lexeme().to_string()));

    let (rest, end_of_line_token) = end_of_line
        .or_fail_with(ParserError::from_missing_end_of_line(&alias))
        .parse(rest)?;

    let span = AstSpan::calc_span_with_whitespace(&from_token, &alias, &end_of_line_token);

    let use_model_node = Node::new(
        span,
        UseModel::new(from_path, subcomponents, inputs, Some(alias)),
    );

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

    let (rest, inputs) = opt(model_inputs).parse(rest)?;

    // for error reporting
    let use_path_span = match subcomponents.last() {
        Some(last) => AstSpan::calc_span(&path, last),
        None => AstSpan::from(&path),
    };
    let use_path_or_inputs_span = inputs
        .as_ref()
        .map(|inputs| AstSpan::from(inputs))
        .unwrap_or(use_path_span);

    let (rest, as_token) = as_
        .or_fail_with(ParserError::use_missing_as(&use_path_or_inputs_span))
        .parse(rest)?;

    let (rest, alias) = identifier
        .or_fail_with(ParserError::use_missing_alias(&as_token))
        .parse(rest)?;
    let alias = Node::new(alias, Identifier::new(alias.lexeme().to_string()));

    let (rest, end_of_line_token) = end_of_line
        .or_fail_with(ParserError::use_missing_end_of_line(&alias))
        .parse(rest)?;

    let span = AstSpan::calc_span_with_whitespace(&use_token, &alias, &end_of_line_token);

    let use_model_node = Node::new(
        span,
        UseModel::new(path, subcomponents, inputs, Some(alias)),
    );

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

/// Parses model inputs (e.g., "(x=1, y=2)")
///
/// Model inputs are optional parameters passed to a model when importing it.
/// They are enclosed in parentheses and consist of a comma-separated list
/// of name-value pairs.
///
/// Examples:
/// - `(x=1)` (single input)
/// - `(x=1, y=2)` (two inputs)
/// - `(width=10, height=20, color='red')` (multiple inputs)
/// - `()` (empty inputs)
///
/// The parser requires:
/// - Opening parenthesis `(`
/// - Zero or more comma-separated model inputs
/// - Closing parenthesis `)`
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns a model input list node containing the parsed inputs, or an error if
/// the input list is malformed (e.g., unclosed parentheses).
fn model_inputs(input: Span) -> Result<ModelInputListNode, ParserError> {
    let (rest, paren_left_span) = paren_left.convert_errors().parse(input)?;
    let (rest, inputs) = separated_list0(comma.convert_errors(), model_input).parse(rest)?;
    let (rest, paren_right_span) = paren_right
        .or_fail_with(ParserError::unclosed_paren(&paren_left_span))
        .parse(rest)?;

    let span = AstSpan::calc_span(&paren_left_span, &paren_right_span);

    Ok((rest, Node::new(span, ModelInputList::new(inputs))))
}

/// Parses a single model input (e.g., "x=1")
///
/// A model input consists of an identifier followed by an equals sign and an expression.
/// The identifier is the parameter name, and the expression is the value to assign to it.
///
/// Examples:
/// - `x=1` (simple numeric value)
/// - `name='John'` (string value)
/// - `enabled=true` (boolean value)
/// - `size=width * height` (expression value)
/// - `config={x: 1, y: 2}` (complex expression)
///
/// The parser requires:
/// - A valid identifier as the parameter name
/// - An equals sign `=`
/// - A valid expression as the parameter value
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns a model input node containing the parsed name-value pair, or an error if
/// the input is malformed (e.g., missing equals sign or invalid expression).
fn model_input(input: Span) -> Result<ModelInputNode, ParserError> {
    let (rest, ident) = identifier.convert_errors().parse(input)?;
    let ident_node = Node::new(ident, Identifier::new(ident.lexeme().to_string()));

    let (rest, equals_span) = equals
        .or_fail_with(ParserError::model_input_missing_equals(&ident_node))
        .parse(rest)?;

    let (rest, value) = parse_expr
        .or_fail_with(ParserError::model_input_missing_value(&equals_span))
        .parse(rest)?;

    let span = AstSpan::calc_span(&ident, &value);

    Ok((rest, Node::new(span, ModelInput::new(ident_node, value))))
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
                    assert_eq!(import_node.path(), "foo");
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
                    assert!(use_model.inputs().is_none());
                    assert_eq!(use_model.alias().unwrap().as_str(), "baz");
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_use_decl_with_inputs() {
            let input = Span::new_extra("use foo.bar(x=1, y=2) as baz\n", Config::default());
            let (rest, decl) = parse(input).unwrap();
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model = use_model_node.node_value();
                    assert_eq!(use_model.model_name().as_str(), "foo");
                    assert_eq!(use_model.subcomponents()[0].as_str(), "bar");
                    assert!(use_model.inputs().is_some());
                    assert_eq!(use_model.alias().unwrap().as_str(), "baz");
                }
                _ => panic!("Expected use declaration with inputs"),
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
                    assert!(use_model.inputs().is_none());
                    assert_eq!(use_model.alias().unwrap().as_str(), "baz");
                }
                _ => panic!("Expected from declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_from_decl_with_inputs() {
            let input = Span::new_extra(
                "from foo.bar use model(x=1, y=2) as baz\n",
                Config::default(),
            );
            let (rest, decl) = parse(input).unwrap();
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model = use_model_node.node_value();
                    assert_eq!(use_model.model_name().as_str(), "foo");
                    assert_eq!(use_model.subcomponents()[0].as_str(), "bar");
                    assert!(use_model.inputs().is_some());
                    assert_eq!(use_model.alias().unwrap().as_str(), "baz");
                }
                _ => panic!("Expected from declaration with inputs"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_parse_complete_import_success() {
            let input = Span::new_extra("import foo\n", Config::default());
            let (rest, decl) = parse_complete(input).unwrap();
            match decl.node_value() {
                Decl::Import(import_node) => {
                    assert_eq!(import_node.path(), "foo");
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
                    assert!(use_model.inputs().is_none());
                    assert_eq!(use_model.alias().unwrap().as_str(), "baz");
                }
                _ => panic!("Expected use declaration"),
            }
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn test_parse_complete_from_success() {
            let input = Span::new_extra("from foo.bar use model(x=1) as baz\n", Config::default());
            let (rest, decl) = parse_complete(input).unwrap();
            match decl.node_value() {
                Decl::UseModel(use_model_node) => {
                    let use_model = use_model_node.node_value();
                    assert_eq!(use_model.model_name().as_str(), "foo");
                    assert_eq!(use_model.subcomponents()[0].as_str(), "bar");
                    assert!(use_model.inputs().is_some());
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
                let expected_import_span = AstSpan::new(0, 6, 6);

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
                let expected_import_span = AstSpan::new(0, 6, 7);

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
                let expected_foo_span = AstSpan::new(7, 10, 10);

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
                let expected_foo_bar_span = AstSpan::new(4, 11, 12);

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
                let expected_as_span = AstSpan::new(12, 14, 14);

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
                // TODO: Could we have a better error message here? "Invalid path identifier"
                let input = Span::new_extra("use 123.bar as baz\n", Config::default());
                let result = parse(input);
                let expected_use_span = AstSpan::new(0, 3, 4);

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
                // TODO: Could we have a better error message here? "Invalid alias identifier"
                let input = Span::new_extra("use foo.bar as 123\n", Config::default());
                let result = parse(input);
                let expected_as_span = AstSpan::new(12, 14, 15);

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

            #[test]
            fn test_unclosed_parentheses() {
                let input = Span::new_extra("use foo.bar(x = 1 as baz\n", Config::default());
                let result = parse(input);
                let expected_paren_span = AstSpan::new(11, 12, 12);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 18);

                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::UnclosedParen,
                                cause,
                            } => {
                                assert_eq!(cause, expected_paren_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_missing_equals_in_input() {
                let input = Span::new_extra("use foo.bar(x 1) as baz\n", Config::default());
                let result = parse(input);
                let expected_x_span = AstSpan::new(12, 13, 14);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 14);

                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Decl(DeclKind::ModelInputMissingEquals),
                                cause,
                            } => {
                                assert_eq!(cause, expected_x_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_invalid_model_input() {
                let input = Span::new_extra("use foo.bar(x = ) as baz\n", Config::default());
                let result = parse(input);
                let expected_equals_span = AstSpan::new(14, 15, 16);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 16);

                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Decl(DeclKind::ModelInputMissingValue),
                                cause,
                            } => {
                                assert_eq!(cause, expected_equals_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_missing_comma_between_inputs() {
                // TODO: Could we have a better error message here?
                let input = Span::new_extra("use foo.bar(x=1 y=2) as baz\n", Config::default());
                let result = parse(input);
                let expected_paren_span = AstSpan::new(11, 12, 12);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 16);

                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::UnclosedParen,
                                cause,
                            } => {
                                assert_eq!(cause, expected_paren_span);
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
                let expected_foo_bar_span = AstSpan::new(5, 12, 13);

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
                let expected_use_span = AstSpan::new(13, 16, 17);

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
                let expected_model_span = AstSpan::new(17, 22, 23);

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
                let expected_as_span = AstSpan::new(23, 25, 25);

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
                let expected_from_span = AstSpan::new(0, 4, 5);

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
                let expected_use_span = AstSpan::new(13, 16, 17);

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
                let expected_as_span = AstSpan::new(23, 25, 26);

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

            #[test]
            fn test_unclosed_parentheses() {
                let input =
                    Span::new_extra("from foo.bar use model(x = 1 as baz\n", Config::default());
                let result = parse(input);
                let expected_paren_span = AstSpan::new(22, 23, 23);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 29);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::UnclosedParen,
                                cause,
                            } => {
                                assert_eq!(cause, expected_paren_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_invalid_model_input() {
                let input =
                    Span::new_extra("from foo.bar use model(x = ) as baz\n", Config::default());
                let result = parse(input);
                let expected_equals_span = AstSpan::new(25, 26, 27);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 27);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Decl(DeclKind::ModelInputMissingValue),
                                cause,
                            } => {
                                assert_eq!(cause, expected_equals_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_missing_equals_in_input() {
                let input =
                    Span::new_extra("from foo.bar use model(x 1) as baz\n", Config::default());
                let result = parse(input);
                let expected_x_span = AstSpan::new(23, 24, 25);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 25);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Decl(DeclKind::ModelInputMissingEquals),
                                cause,
                            } => {
                                assert_eq!(cause, expected_x_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_missing_comma_between_inputs() {
                let input = Span::new_extra(
                    "from foo.bar use model(x=1 y=2) as baz\n",
                    Config::default(),
                );
                let result = parse(input);
                let expected_paren_span = AstSpan::new(22, 23, 23);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 27);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::UnclosedParen,
                                cause,
                            } => {
                                assert_eq!(cause, expected_paren_span);
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
                let expected_dot_span = AstSpan::new(3, 4, 4);

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
                let expected_dot_span = AstSpan::new(3, 4, 4);

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
                let expected_dot_span = AstSpan::new(3, 4, 4);

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
                let expected_dot_span = AstSpan::new(7, 8, 8);

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

        mod model_inputs_error_tests {
            use crate::error::reason::{DeclKind, IncompleteKind, ParserErrorReason};
            use crate::token::error::{ExpectKind, ExpectSymbol, TokenErrorKind};

            use super::*;

            #[test]
            fn test_missing_opening_paren() {
                let input = Span::new_extra("x = 1)", Config::default());
                let result = model_inputs(input);
                match result {
                    Err(nom::Err::Error(error)) => {
                        assert_eq!(error.error_offset, 0);
                        assert!(matches!(
                            error.reason,
                            ParserErrorReason::TokenError(TokenErrorKind::Expect(
                                ExpectKind::Symbol(ExpectSymbol::ParenLeft)
                            ))
                        ));
                    }
                    _ => panic!("Expected error for missing opening paren"),
                }
            }

            #[test]
            fn test_missing_closing_paren() {
                let input = Span::new_extra("(x = 1", Config::default());
                let result = model_inputs(input);
                let expected_paren_span = AstSpan::new(0, 1, 1);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 6);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::UnclosedParen,
                                cause,
                            } => {
                                assert_eq!(cause, expected_paren_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_empty_parentheses() {
                let input = Span::new_extra("()", Config::default());
                let (rest, inputs) = model_inputs(input).unwrap();
                assert_eq!(inputs.node_value().inputs().len(), 0);
                assert_eq!(rest.fragment(), &"");
            }

            #[test]
            fn test_missing_equals_in_input() {
                let input = Span::new_extra("(x 1)", Config::default());
                let result = model_inputs(input);
                let expected_x_span = AstSpan::new(1, 2, 3);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 3);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Decl(DeclKind::ModelInputMissingEquals),
                                cause,
                            } => {
                                assert_eq!(cause, expected_x_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_missing_value_in_input() {
                let input = Span::new_extra("(x = )", Config::default());
                let result = model_inputs(input);
                let expected_equals_span = AstSpan::new(3, 4, 5);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 5);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Decl(DeclKind::ModelInputMissingValue),
                                cause,
                            } => {
                                assert_eq!(cause, expected_equals_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_missing_comma_between_inputs() {
                let input = Span::new_extra("(x = 1 y = 2)", Config::default());
                let result = model_inputs(input);
                let expected_paren_span = AstSpan::new(0, 1, 1);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 7);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::UnclosedParen,
                                cause,
                            } => {
                                assert_eq!(cause, expected_paren_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_trailing_comma() {
                // TODO: Improve this error: "missing test input"
                let input = Span::new_extra("(x = 1,)", Config::default());
                let result = model_inputs(input);
                let expected_paren_span = AstSpan::new(0, 1, 1);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 6);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::UnclosedParen,
                                cause,
                            } => {
                                assert_eq!(cause, expected_paren_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_invalid_identifier_in_input() {
                // TODO: Improve this error: "invalid identifier"
                let input = Span::new_extra("(123 = 1)", Config::default());
                let result = model_inputs(input);
                let expected_paren_span = AstSpan::new(0, 1, 1);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 1);

                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::UnclosedParen,
                                cause,
                            } => {
                                assert_eq!(cause, expected_paren_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }
        }

        mod model_input_error_tests {
            use crate::error::reason::{DeclKind, IncompleteKind, ParserErrorReason};
            use crate::token::error::{ExpectKind, TokenErrorKind};

            use super::*;

            #[test]
            fn test_empty_input() {
                let input = Span::new_extra("", Config::default());
                let result = model_input(input);
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
                    _ => panic!("Expected error for empty input"),
                }
            }

            #[test]
            fn test_missing_identifier() {
                let input = Span::new_extra("= 1", Config::default());
                let result = model_input(input);
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
                    _ => panic!("Expected error for missing identifier"),
                }
            }

            #[test]
            fn test_missing_equals() {
                let input = Span::new_extra("x 1", Config::default());
                let result = model_input(input);
                let expected_x_span = AstSpan::new(0, 1, 2);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 2);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Decl(DeclKind::ModelInputMissingEquals),
                                cause,
                            } => {
                                assert_eq!(cause, expected_x_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_missing_value() {
                let input = Span::new_extra("x =", Config::default());
                let result = model_input(input);
                let expected_equals_span = AstSpan::new(2, 3, 3);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 3);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Decl(DeclKind::ModelInputMissingValue),
                                cause,
                            } => {
                                assert_eq!(cause, expected_equals_span);
                            }
                            _ => panic!("Unexpected reason {:?}", error.reason),
                        }
                    }
                    _ => panic!("Unexpected result {:?}", result),
                }
            }

            #[test]
            fn test_invalid_identifier() {
                // TODO: Improve this error: "invalid identifier"
                let input = Span::new_extra("123 = 1", Config::default());
                let result = model_input(input);
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
                    _ => panic!("Expected error for invalid identifier"),
                }
            }

            #[test]
            fn test_invalid_value_expression() {
                // TODO: Improve this error: "invalid value expression"
                let input = Span::new_extra("x = @", Config::default());
                let result = model_input(input);
                let expected_equals_span = AstSpan::new(2, 3, 4);

                match result {
                    Err(nom::Err::Failure(error)) => {
                        assert_eq!(error.error_offset, 4);
                        match error.reason {
                            ParserErrorReason::Incomplete {
                                kind: IncompleteKind::Decl(DeclKind::ModelInputMissingValue),
                                cause,
                            } => {
                                assert_eq!(cause, expected_equals_span);
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
