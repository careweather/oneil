//! Parser for declarations in an Oneil program.

use nom::{
    Parser as _,
    branch::alt,
    combinator::{all_consuming, cut, opt},
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

fn decl(input: Span) -> Result<DeclNode, ParserError> {
    alt((import_decl, from_decl, use_decl, parameter_decl, test_decl))
        .or_fail_with(ParserError::expect_decl)
        .parse(input)
}

/// Parses an import declaration
fn import_decl(input: Span) -> Result<DeclNode, ParserError> {
    let (rest, import_token) = import.convert_errors().parse(input)?;

    let (rest, path) =
        cut(identifier.or_fail_with(ParserError::import_missing_path(&import_token)))
            .parse(rest)?;

    let (rest, end_of_line_token) =
        cut(end_of_line.or_fail_with(ParserError::import_missing_end_of_line(&import_token)))
            .parse(rest)?;

    let span = AstSpan::calc_span_with_whitespace(&import_token, &path, &end_of_line_token);

    let import_node = Node::new(span, Import::new(path.lexeme().to_string()));

    let decl_node = Node::new(span, Decl::Import(import_node));

    Ok((rest, decl_node))
}

/// Parses a from declaration
fn from_decl(input: Span) -> Result<DeclNode, ParserError> {
    let (rest, from_token) = from.convert_errors().parse(input)?;

    let (rest, (path, mut subcomponents)) = model_path
        .or_fail_with(ParserError::from_missing_path(&from_token))
        .parse(rest)?;

    let (rest, use_token) = use_
        .or_fail_with(ParserError::from_missing_use(&from_token))
        .parse(rest)?;

    let (rest, use_model) = identifier
        .or_fail_with(ParserError::from_missing_use_model(&from_token, &use_token))
        .parse(rest)?;
    let use_model = Node::new(use_model, Identifier::new(use_model.lexeme().to_string()));
    subcomponents.push(use_model);

    let (rest, inputs) = opt(model_inputs).parse(rest)?;

    let (rest, as_token) = as_
        .or_fail_with(ParserError::from_missing_as(&from_token))
        .parse(rest)?;

    let (rest, alias) = identifier
        .or_fail_with(ParserError::from_missing_alias(&as_token))
        .parse(rest)?;
    let alias = Node::new(alias, Identifier::new(alias.lexeme().to_string()));

    let (rest, end_of_line_token) = end_of_line
        .or_fail_with(ParserError::from_missing_end_of_line(&from_token))
        .parse(rest)?;

    let span = AstSpan::calc_span_with_whitespace(&from_token, &alias, &end_of_line_token);

    let use_model_node = Node::new(
        span,
        UseModel::new(path, subcomponents, inputs, Some(alias)),
    );

    let decl_node = Node::new(span, Decl::UseModel(use_model_node));

    Ok((rest, decl_node))
}

/// Parses a use declaration
fn use_decl(input: Span) -> Result<DeclNode, ParserError> {
    let (rest, use_token) = use_.convert_errors().parse(input)?;

    let (rest, (path, subcomponents)) = model_path
        .or_fail_with(ParserError::use_missing_path(&use_token))
        .parse(rest)?;

    let (rest, inputs) = opt(model_inputs).parse(rest)?;

    let (rest, as_token) = as_
        .or_fail_with(ParserError::use_missing_as(&use_token))
        .parse(rest)?;

    let (rest, alias) = identifier
        .or_fail_with(ParserError::use_missing_alias(&as_token))
        .parse(rest)?;
    let alias = Node::new(alias, Identifier::new(alias.lexeme().to_string()));

    let (rest, end_of_line_token) = end_of_line
        .or_fail_with(ParserError::use_missing_end_of_line(&use_token))
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
fn model_path(input: Span) -> Result<(IdentifierNode, Vec<IdentifierNode>), ParserError> {
    let (rest, path) = identifier.convert_errors().parse(input)?;
    let path = Node::new(path, Identifier::new(path.lexeme().to_string()));

    let (rest, subcomponents) = many0(|input| {
        let (rest, dot_token) = dot.convert_errors().parse(input)?;
        let (rest, subcomponent) =
            cut(identifier.or_fail_with(ParserError::model_path_missing_subcomponent(&dot_token)))
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
fn model_input(input: Span) -> Result<ModelInputNode, ParserError> {
    let (rest, ident) = identifier.convert_errors().parse(input)?;
    let ident_node = Node::new(ident, Identifier::new(ident.lexeme().to_string()));

    let (rest, equals_span) = equals.convert_errors().parse(rest)?;
    let (rest, value) = parse_expr
        .or_fail_with(ParserError::model_input_missing_value(&ident, &equals_span))
        .parse(rest)?;

    let span = AstSpan::calc_span(&ident, &value);

    Ok((rest, Node::new(span, ModelInput::new(ident_node, value))))
}

fn parameter_decl(input: Span) -> Result<DeclNode, ParserError> {
    let (rest, parameter) = parse_parameter.parse(input)?;

    let span = AstSpan::from(&parameter);
    let decl_node = Node::new(span, Decl::Parameter(parameter));

    Ok((rest, decl_node))
}

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
