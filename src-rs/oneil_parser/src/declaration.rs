//! Parser for declarations in an Oneil program.

use nom::{
    Parser as _,
    branch::alt,
    combinator::{all_consuming, cut, map, opt},
    multi::{many0, separated_list0},
};

use oneil_ast::declaration::{Decl, ModelInput};

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
pub fn parse(input: Span) -> Result<Decl, ParserError> {
    decl.parse(input)
}

/// Parses a declaration
///
/// This function **fails if the complete input is not consumed**.
pub fn parse_complete(input: Span) -> Result<Decl, ParserError> {
    all_consuming(decl).parse(input)
}

fn decl(input: Span) -> Result<Decl, ParserError> {
    alt((
        import_decl,
        from_decl,
        use_decl,
        map(parse_parameter, Decl::Parameter),
        map(parse_test, Decl::Test),
    ))
    .map_error(ParserError::expect_decl)
    .parse(input)
}

/// Parses an import declaration
fn import_decl(input: Span) -> Result<Decl, ParserError> {
    let (rest, import_token) = import.convert_errors().parse(input)?;

    let (rest, path) = cut(identifier)
        .map_failure(ParserError::import_missing_path(import_token))
        .parse(rest)?;

    let (rest, _) = cut(end_of_line)
        .map_failure(ParserError::import_missing_end_of_line(import_token))
        .parse(rest)?;

    Ok((
        rest,
        Decl::Import {
            path: path.lexeme().to_string(),
        },
    ))
}

/// Parses a from declaration
fn from_decl(input: Span) -> Result<Decl, ParserError> {
    let (rest, from_token) = from.convert_errors().parse(input)?;

    let (rest, (model_name, mut subcomponents)) = cut(module_path)
        .map_failure(ParserError::from_missing_path(from_token))
        .parse(rest)?;

    let (rest, use_token) = cut(use_)
        .map_failure(ParserError::from_missing_use(from_token))
        .parse(rest)?;

    let (rest, use_model) = cut(identifier)
        .map_failure(ParserError::from_missing_use_model(from_token, use_token))
        .parse(rest)?;
    subcomponents.push(use_model.lexeme().to_string());

    let (rest, inputs) = opt(model_inputs).parse(rest)?;

    let (rest, as_token) = cut(as_)
        .map_failure(ParserError::from_missing_as(from_token))
        .parse(rest)?;

    let (rest, as_name) = cut(identifier)
        .map_failure(ParserError::from_missing_as_name(as_token))
        .parse(rest)?;

    let (rest, _) = cut(end_of_line)
        .map_failure(ParserError::from_missing_end_of_line(from_token))
        .parse(rest)?;

    Ok((
        rest,
        Decl::UseModel {
            model_name,
            subcomponents,
            inputs,
            as_name: Some(as_name.lexeme().to_string()),
        },
    ))
}

/// Parses a use declaration
fn use_decl(input: Span) -> Result<Decl, ParserError> {
    let (rest, use_span) = use_.convert_errors().parse(input)?;

    let (rest, (model_name, subcomponents)) = cut(module_path)
        .map_failure(ParserError::use_missing_path(use_span))
        .parse(rest)?;

    let (rest, inputs) = opt(model_inputs).parse(rest)?;

    let (rest, as_token) = cut(as_)
        .map_failure(ParserError::use_missing_as(use_span))
        .parse(rest)?;

    let (rest, as_name) = cut(identifier)
        .map_failure(ParserError::use_missing_as_name(as_token))
        .parse(rest)?;

    let (rest, _) = cut(end_of_line)
        .map_failure(ParserError::use_missing_end_of_line(use_span))
        .parse(rest)?;

    Ok((
        rest,
        Decl::UseModel {
            model_name,
            subcomponents,
            inputs,
            as_name: Some(as_name.lexeme().to_string()),
        },
    ))
}

/// Parses a module path (e.g., "foo.bar.baz")
fn module_path(input: Span) -> Result<(String, Vec<String>), ParserError> {
    let (rest, path) = identifier.convert_errors().parse(input)?;
    let (rest, subcomponents) = many0(|input| {
        let (rest, dot_token) = dot.convert_errors().parse(input)?;
        let (rest, subcomponent) =
            cut(identifier.map_error(ParserError::module_path_missing_subcomponent(dot_token)))
                .parse(rest)?;
        Ok((rest, subcomponent.lexeme().to_string()))
    })
    .parse(rest)?;

    Ok((rest, (path.lexeme().to_string(), subcomponents)))
}

/// Parses model inputs (e.g., "(x=1, y=2)")
fn model_inputs(input: Span) -> Result<Vec<ModelInput>, ParserError> {
    let (rest, paren_left_span) = paren_left.convert_errors().parse(input)?;
    let (rest, inputs) = separated_list0(comma.convert_errors(), model_input).parse(rest)?;
    let (rest, _) = paren_right
        .map_failure(ParserError::unclosed_paren(paren_left_span))
        .parse(rest)?;

    Ok((rest, inputs))
}

/// Parses a single model input (e.g., "x=1")
fn model_input(input: Span) -> Result<ModelInput, ParserError> {
    let (rest, name) = identifier.convert_errors().parse(input)?;
    let (rest, equals_span) = equals.convert_errors().parse(rest)?;
    let (rest, value) = cut(parse_expr)
        .map_failure(ParserError::model_input_missing_value(name, equals_span))
        .parse(rest)?;

    Ok((
        rest,
        ModelInput {
            name: name.lexeme().to_string(),
            value,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;
    use oneil_ast::expression::{Expr, Literal};

    #[test]
    fn test_import_decl() {
        let input = Span::new_extra("import foo\n", Config::default());
        let (rest, decl) = parse(input).unwrap();
        match decl {
            Decl::Import { path } => {
                assert_eq!(path, "foo");
            }
            _ => panic!("Expected import declaration"),
        }
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_use_decl() {
        let input = Span::new_extra("use foo.bar as baz\n", Config::default());
        let (rest, decl) = parse(input).unwrap();
        match decl {
            Decl::UseModel {
                model_name,
                subcomponents,
                inputs,
                as_name,
            } => {
                assert_eq!(model_name, "foo");
                assert_eq!(subcomponents, ["bar"]);
                assert!(inputs.is_none());
                assert_eq!(as_name, Some("baz".to_string()));
            }
            _ => panic!("Expected use declaration"),
        }
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_use_decl_with_inputs() {
        let input = Span::new_extra("use foo.bar(x=1, y=2) as baz\n", Config::default());
        let (rest, decl) = parse(input).unwrap();
        match decl {
            Decl::UseModel {
                model_name,
                subcomponents,
                inputs: Some(inputs),
                as_name,
            } => {
                assert_eq!(model_name, "foo");
                assert_eq!(subcomponents, ["bar"]);
                assert_eq!(inputs.len(), 2);
                assert_eq!(inputs[0].name, "x");
                assert_eq!(inputs[0].value, Expr::Literal(Literal::Number(1.0)));
                assert_eq!(inputs[1].name, "y");
                assert_eq!(inputs[1].value, Expr::Literal(Literal::Number(2.0)));
                assert_eq!(as_name, Some("baz".to_string()));
            }
            _ => panic!("Expected use declaration with inputs"),
        }
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_from_decl() {
        let input = Span::new_extra("from foo.bar use model as baz\n", Config::default());
        let (rest, decl) = parse(input).unwrap();
        match decl {
            Decl::UseModel {
                model_name,
                subcomponents,
                inputs,
                as_name,
            } => {
                assert_eq!(model_name, "foo");
                assert_eq!(subcomponents, ["bar", "model"]);
                assert!(inputs.is_none());
                assert_eq!(as_name, Some("baz".to_string()));
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
        match decl {
            Decl::UseModel {
                model_name,
                subcomponents,
                inputs: Some(inputs),
                as_name,
            } => {
                assert_eq!(model_name, "foo");
                assert_eq!(subcomponents, ["bar", "model"]);
                assert_eq!(inputs.len(), 2);
                assert_eq!(inputs[0].name, "x");
                assert_eq!(inputs[0].value, Expr::Literal(Literal::Number(1.0)));
                assert_eq!(inputs[1].name, "y");
                assert_eq!(inputs[1].value, Expr::Literal(Literal::Number(2.0)));
                assert_eq!(as_name, Some("baz".to_string()));
            }
            _ => panic!("Expected from declaration with inputs"),
        }
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_parse_complete_import_success() {
        let input = Span::new_extra("import foo\n", Config::default());
        let (rest, decl) = parse_complete(input).unwrap();
        match decl {
            Decl::Import { path } => {
                assert_eq!(path, "foo");
            }
            _ => panic!("Expected import declaration"),
        }
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_parse_complete_use_success() {
        let input = Span::new_extra("use foo.bar as baz\n", Config::default());
        let (rest, decl) = parse_complete(input).unwrap();
        match decl {
            Decl::UseModel {
                model_name,
                subcomponents,
                inputs,
                as_name,
            } => {
                assert_eq!(model_name, "foo");
                assert_eq!(subcomponents, ["bar"]);
                assert!(inputs.is_none());
                assert_eq!(as_name, Some("baz".to_string()));
            }
            _ => panic!("Expected use declaration"),
        }
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_parse_complete_from_success() {
        let input = Span::new_extra("from foo.bar use model(x=1) as baz\n", Config::default());
        let (rest, decl) = parse_complete(input).unwrap();
        match decl {
            Decl::UseModel {
                model_name,
                subcomponents,
                inputs: Some(inputs),
                as_name,
            } => {
                assert_eq!(model_name, "foo");
                assert_eq!(subcomponents, ["bar", "model"]);
                assert_eq!(inputs.len(), 1);
                assert_eq!(inputs[0].name, "x");
                assert_eq!(inputs[0].value, Expr::Literal(Literal::Number(1.0)));
                assert_eq!(as_name, Some("baz".to_string()));
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
