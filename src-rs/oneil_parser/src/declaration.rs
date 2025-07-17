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

    let (rest, use_token) = use_
        .or_fail_with(ParserError::from_missing_use(&from_path))
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
    let use_path_or_inputs_span = inputs
        .as_ref()
        .map(|inputs| AstSpan::from(inputs))
        .or_else(|| subcomponents.last().map(AstSpan::from))
        .unwrap_or(AstSpan::from(&path));

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

    let (rest, equals_span) = equals.convert_errors().parse(rest)?;
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

    #[test]
    fn test_parse_complete_with_remaining_input() {
        let input = Span::new_extra("import foo\nrest", Config::default());
        let result = parse_complete(input);
        assert!(result.is_err());
    }
}
