//! Parser for declarations in an Oneil program.
//!
//! # Examples
//!
//! ```
//! use oneil::parser::declaration::parse;
//! use oneil::parser::{Config, Span};
//!
//! // Parse an import declaration
//! let input = Span::new_extra("import foo\n", Config::default());
//! let (_, decl) = parse(input).unwrap();
//!
//! // Parse a use declaration
//! let input = Span::new_extra("use foo.bar as baz\n", Config::default());
//! let (_, decl) = parse(input).unwrap();
//! ```

use nom::{
    Parser as _,
    branch::alt,
    combinator::{all_consuming, cut, map, opt},
    multi::separated_list1,
};

use crate::ast::declaration::{Decl, ModelInput};

use super::{
    error::{ErrorHandlingParser as _, ParserError, ParserErrorKind},
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
///
/// # Examples
///
/// ```
/// use oneil::parser::declaration::parse;
/// use oneil::parser::{Config, Span};
///
/// let input = Span::new_extra("import foo\n", Config::default());
/// let (rest, decl) = parse(input).unwrap();
/// assert_eq!(rest.fragment(), &"");
/// ```
///
/// ```
/// use oneil::parser::declaration::parse;
/// use oneil::parser::{Config, Span};
///
/// let input = Span::new_extra("import foo\nrest", Config::default());
/// let (rest, decl) = parse(input).unwrap();
/// assert_eq!(rest.fragment(), &"rest");
/// ```
pub fn parse(input: Span) -> Result<Decl, ParserError> {
    decl.parse(input)
}

/// Parses a declaration
///
/// This function **fails if the complete input is not consumed**.
///
/// # Examples
///
/// ```
/// use oneil::parser::declaration::parse_complete;
/// use oneil::parser::{Config, Span};
///
/// let input = Span::new_extra("import foo\n", Config::default());
/// let (rest, decl) = parse_complete(input).unwrap();
/// assert_eq!(rest.fragment(), &"");
/// ```
///
/// ```
/// use oneil::parser::declaration::parse_complete;
/// use oneil::parser::{Config, Span};
///
/// let input = Span::new_extra("import foo\nrest", Config::default());
/// let result = parse_complete(input);
/// assert_eq!(result.is_err(), true);
/// ```
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
    .map_error(|e| ParserError::new(ParserErrorKind::ExpectDecl, e.span))
    .parse(input)
}

/// Parses an import declaration
fn import_decl(input: Span) -> Result<Decl, ParserError> {
    (import, cut((identifier, end_of_line)))
        .map(|(_, (path, _))| Decl::Import {
            path: path.to_string(),
        })
        .convert_errors()
        .parse(input)
}

/// Parses a from declaration
fn from_decl(input: Span) -> Result<Decl, ParserError> {
    (
        from.convert_errors(),
        cut((
            module_path,
            use_.convert_errors(),
            identifier.convert_errors(),
            opt(model_inputs),
            as_.convert_errors(),
            identifier.convert_errors(),
            end_of_line.convert_errors(),
        )),
    )
        .map(
            |(_, (path, _, use_model, inputs, _, as_name, _))| Decl::From {
                path: path.to_string(),
                use_model: use_model.to_string(),
                inputs,
                as_name: as_name.to_string(),
            },
        )
        .parse(input)
}

/// Parses a use declaration
fn use_decl(input: Span) -> Result<Decl, ParserError> {
    (
        use_.convert_errors(),
        cut((
            module_path,
            opt(model_inputs),
            as_.convert_errors(),
            identifier.convert_errors(),
            end_of_line.convert_errors(),
        )),
    )
        .map(|(_, (path, inputs, _, as_name, _))| Decl::Use {
            path: path.to_string(),
            inputs,
            as_name: as_name.to_string(),
        })
        .parse(input)
}

/// Parses a module path (e.g., "foo.bar.baz")
fn module_path(input: Span) -> Result<String, ParserError> {
    separated_list1(dot, identifier)
        .map(|parts| {
            parts
                .into_iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(".")
        })
        .convert_errors()
        .parse(input)
}

/// Parses model inputs (e.g., "(x=1, y=2)")
fn model_inputs(input: Span) -> Result<Vec<ModelInput>, ParserError> {
    (
        paren_left.convert_errors(),
        cut((
            separated_list1(comma.convert_errors(), model_input),
            paren_right.convert_errors(),
        )),
    )
        .map(|(_, (inputs, _))| inputs)
        .parse(input)
}

/// Parses a single model input (e.g., "x=1")
fn model_input(input: Span) -> Result<ModelInput, ParserError> {
    (
        identifier.convert_errors(),
        equals.convert_errors(),
        cut(parse_expr),
    )
        .map(|(name, _, value)| ModelInput {
            name: name.to_string(),
            value,
        })
        .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expression::{Expr, Literal};
    use crate::parser::Config;

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
            Decl::Use {
                path,
                inputs,
                as_name,
            } => {
                assert_eq!(path, "foo.bar");
                assert!(inputs.is_none());
                assert_eq!(as_name, "baz");
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
            Decl::Use {
                path,
                inputs: Some(inputs),
                as_name,
            } => {
                assert_eq!(path, "foo.bar");
                assert_eq!(inputs.len(), 2);
                assert_eq!(inputs[0].name, "x");
                assert_eq!(inputs[0].value, Expr::Literal(Literal::Number(1.0)));
                assert_eq!(inputs[1].name, "y");
                assert_eq!(inputs[1].value, Expr::Literal(Literal::Number(2.0)));
                assert_eq!(as_name, "baz");
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
            Decl::From {
                path,
                use_model,
                inputs,
                as_name,
            } => {
                assert_eq!(path, "foo.bar");
                assert_eq!(use_model, "model");
                assert!(inputs.is_none());
                assert_eq!(as_name, "baz");
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
            Decl::From {
                path,
                use_model,
                inputs: Some(inputs),
                as_name,
            } => {
                assert_eq!(path, "foo.bar");
                assert_eq!(use_model, "model");
                assert_eq!(inputs.len(), 2);
                assert_eq!(inputs[0].name, "x");
                assert_eq!(inputs[0].value, Expr::Literal(Literal::Number(1.0)));
                assert_eq!(inputs[1].name, "y");
                assert_eq!(inputs[1].value, Expr::Literal(Literal::Number(2.0)));
                assert_eq!(as_name, "baz");
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
            Decl::Use {
                path,
                inputs,
                as_name,
            } => {
                assert_eq!(path, "foo.bar");
                assert!(inputs.is_none());
                assert_eq!(as_name, "baz");
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
            Decl::From {
                path,
                use_model,
                inputs: Some(inputs),
                as_name,
            } => {
                assert_eq!(path, "foo.bar");
                assert_eq!(use_model, "model");
                assert_eq!(inputs.len(), 1);
                assert_eq!(inputs[0].name, "x");
                assert_eq!(inputs[0].value, Expr::Literal(Literal::Number(1.0)));
                assert_eq!(as_name, "baz");
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
