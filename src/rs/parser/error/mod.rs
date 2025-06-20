//! Error handling for the Oneil language parser.
//!
//! This module provides a comprehensive error handling system for the parser,
//! including:
//!
//! - A trait for consistent error handling across parser components
//! - Error types that capture both the type of error and its location
//! - Conversion functions between different error types
//!
//! The error system is built on top of nom's error handling, extending it with
//! Oneil-specific error types and location tracking.
//!
//! # Error Handling Strategy
//!
//! The parser uses a two-level error handling approach:
//!
//! 1. Token-level errors (`TokenError`): For low-level parsing issues like
//!    invalid characters or unterminated strings
//! 2. Parser-level errors (`ParserError`): For higher-level issues like
//!    invalid syntax or unexpected tokens

use nom::error::{FromExternalError, ParseError};

use super::{
    Span,
    token::{
        Token,
        error::{TokenError, TokenErrorKind},
    },
};

use crate::ast::expression::{BinaryOp, UnaryOp};

mod parser_trait;
pub use parser_trait::ErrorHandlingParser;

pub mod partial;

/// An error that occurred during parsing.
///
/// This type represents high-level parsing errors, containing both the specific
/// kind of error and the location where it occurred. It is used for errors that
/// occur during the parsing of language constructs like declarations, expressions,
/// and parameters.
///
/// # Examples
///
/// ```
/// use oneil::parser::error::{ParserError, ParserErrorKind};
/// use oneil::parser::{Config, Span};
///
/// // Create an error for an invalid expression
/// let span = Span::new_extra("1 + ", Config::default());
/// let error = ParserError::new(ParserErrorKind::ExpectExpr, span);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ParserError {
    /// The specific kind of error that occurred
    pub kind: ParserErrorKind,
    /// The location in the source where the error occurred
    pub offset: usize,
}

impl ParserError {
    /// Creates a new parser error with the given kind and location.
    pub fn new(kind: ParserErrorKind, span: Span) -> Self {
        Self {
            kind,
            offset: span.location_offset(),
        }
    }

    /// Converts the error kind to a new kind
    ///
    /// This is used to convert a wrapped token error to a parser error
    fn convert_kind(self, kind: ParserErrorKind) -> Self {
        let is_token_error = matches!(
            self.kind,
            ParserErrorKind::TokenError(TokenErrorKind::Expect(_))
        );
        assert!(
            is_token_error,
            "Cannot convert a non-token error to a parser error (attempted to convert {:?})",
            self.kind
        );

        Self { kind, ..self }
    }

    /// Creates a new ParserError for an expected declaration
    pub fn expect_decl(error: Self) -> Self {
        Self {
            kind: ParserErrorKind::Expect(ExpectKind::Decl),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for an expected expression
    pub fn expect_expr(error: Self) -> Self {
        Self {
            kind: ParserErrorKind::Expect(ExpectKind::Expr),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for an expected note
    pub fn expect_note(error: TokenError) -> Self {
        Self {
            kind: ParserErrorKind::Expect(ExpectKind::Note),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for an expected parameter
    pub fn expect_parameter(error: TokenError) -> Self {
        Self {
            kind: ParserErrorKind::Expect(ExpectKind::Parameter),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for an expected test
    pub fn expect_test(error: TokenError) -> Self {
        Self {
            kind: ParserErrorKind::Expect(ExpectKind::Test),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for an expected unit
    pub fn expect_unit(error: Self) -> Self {
        Self {
            kind: ParserErrorKind::Expect(ExpectKind::Unit),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing path in an import declaration
    pub fn import_missing_path(import_token: Token) -> impl Fn(TokenError) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Decl(DeclKind::Import {
                import_offset: import_token.lexeme_offset(),
                kind: ImportKind::MissingPath,
            })),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing end of line in an import declaration
    pub fn import_missing_end_of_line(import_token: Token) -> impl Fn(TokenError) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Decl(DeclKind::Import {
                import_offset: import_token.lexeme_offset(),
                kind: ImportKind::MissingEndOfLine,
            })),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing path in a from declaration
    pub fn from_missing_path(from_token: Token) -> impl Fn(Self) -> Self {
        move |error| {
            error.convert_kind(ParserErrorKind::Incomplete(IncompleteKind::Decl(
                DeclKind::From {
                    from_offset: from_token.lexeme_offset(),
                    kind: FromKind::MissingPath,
                },
            )))
        }
    }

    /// Creates a new ParserError for a missing use keyword in a from declaration
    pub fn from_missing_use(from_token: Token) -> impl Fn(TokenError) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Decl(DeclKind::From {
                from_offset: from_token.lexeme_offset(),
                kind: FromKind::MissingUse,
            })),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing use model in a from declaration
    pub fn from_missing_use_model(
        from_token: Token,
        use_token: Token,
    ) -> impl Fn(TokenError) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Decl(DeclKind::From {
                from_offset: from_token.lexeme_offset(),
                kind: FromKind::MissingUseModel {
                    use_offset: use_token.lexeme_offset(),
                },
            })),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing as keyword in a from declaration
    pub fn from_missing_as(from_token: Token) -> impl Fn(TokenError) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Decl(DeclKind::From {
                from_offset: from_token.lexeme_offset(),
                kind: FromKind::MissingAs,
            })),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing as name in a from declaration
    pub fn from_missing_as_name(as_token: Token) -> impl Fn(TokenError) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Decl(DeclKind::From {
                from_offset: as_token.lexeme_offset(),
                kind: FromKind::MissingAsName,
            })),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing end of line in a from declaration
    pub fn from_missing_end_of_line(from_token: Token) -> impl Fn(TokenError) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Decl(DeclKind::From {
                from_offset: from_token.lexeme_offset(),
                kind: FromKind::MissingEndOfLine,
            })),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing path in a use declaration
    pub fn use_missing_path(use_token: Token) -> impl Fn(Self) -> Self {
        move |error| {
            error.convert_kind(ParserErrorKind::Incomplete(IncompleteKind::Decl(
                DeclKind::Use {
                    use_offset: use_token.lexeme_offset(),
                    kind: UseKind::MissingPath,
                },
            )))
        }
    }

    /// Creates a new ParserError for a missing as keyword in a use declaration
    pub fn use_missing_as(use_token: Token) -> impl Fn(TokenError) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Decl(DeclKind::Use {
                use_offset: use_token.lexeme_offset(),
                kind: UseKind::MissingAs,
            })),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing as name in a use declaration
    pub fn use_missing_as_name(as_token: Token) -> impl Fn(TokenError) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Decl(DeclKind::Use {
                use_offset: as_token.lexeme_offset(),
                kind: UseKind::MissingAsName,
            })),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing end of line in a use declaration
    pub fn use_missing_end_of_line(use_token: Token) -> impl Fn(TokenError) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Decl(DeclKind::Use {
                use_offset: use_token.lexeme_offset(),
                kind: UseKind::MissingEndOfLine,
            })),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing value in a model input
    pub fn model_input_missing_value(name: Token, equals_token: Token) -> impl Fn(Self) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Decl(
                DeclKind::ModelInputMissingValue {
                    identifier: name.lexeme().to_string(),
                    identifier_offset: name.lexeme_offset(),
                    equals_offset: equals_token.lexeme_offset(),
                },
            )),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for a binary operation missing its second operand
    pub fn binary_op_missing_second_operand(operator: BinaryOp) -> impl Fn(Self) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Expr(
                ExprKind::BinaryOpMissingSecondOperand {
                    operator,
                    operator_offset: error.offset,
                },
            )),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for a unary operation missing its operand
    pub fn unary_op_missing_operand(operator: UnaryOp) -> impl Fn(Self) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Expr(
                ExprKind::UnaryOpMissingOperand {
                    operator,
                    operator_offset: error.offset,
                },
            )),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for an invalid number
    pub fn invalid_number(number_token: Token) -> impl Fn() -> Self {
        move || Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::InvalidNumber(
                number_token.lexeme().to_string(),
            )),
            offset: number_token.lexeme_offset(),
        }
    }

    /// Creates a new ParserError for a parenthesis missing its expression
    pub fn paren_missing_expression(paren_left_token: Token) -> impl Fn(Self) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::UnclosedParen {
                paren_left_offset: paren_left_token.lexeme_offset(),
            }),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for an unclosed parenthesis
    pub fn unclosed_paren(paren_left_token: Token) -> impl Fn(TokenError) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::UnclosedParen {
                paren_left_offset: paren_left_token.lexeme_offset(),
            }),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for a section missing a label
    pub fn section_missing_label(section_token: Token) -> impl Fn(TokenError) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Section(SectionKind::MissingLabel {
                section_offset: section_token.lexeme_offset(),
            })),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for a section missing an end of line
    pub fn section_missing_end_of_line(section_token: Token) -> impl Fn(TokenError) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Section(
                SectionKind::MissingEndOfLine {
                    section_offset: section_token.lexeme_offset(),
                },
            )),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing identifier in a parameter
    pub fn parameter_missing_identifier(error: TokenError) -> Self {
        Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Parameter(
                ParameterKind::MissingIdentifier,
            )),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing equals sign in a parameter
    pub fn parameter_missing_equals_sign(ident: Token) -> impl Fn(TokenError) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Parameter(
                ParameterKind::MissingEqualsSign {
                    identifier_offset: ident.lexeme_offset(),
                    identifier_end_offset: ident.get_lexeme_end_offset(),
                },
            )),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing value in a parameter
    pub fn parameter_missing_value(ident: Token) -> impl Fn(Self) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Parameter(
                ParameterKind::MissingValue {
                    identifier_offset: ident.lexeme_offset(),
                    identifier_end_offset: ident.get_lexeme_end_offset(),
                },
            )),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing end of line in a parameter
    pub fn parameter_missing_end_of_line(ident: Token) -> impl Fn(TokenError) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Parameter(
                ParameterKind::MissingEndOfLine {
                    identifier_offset: ident.lexeme_offset(),
                    identifier_end_offset: ident.get_lexeme_end_offset(),
                },
            )),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing unit in a parameter
    pub fn parameter_missing_unit(colon_token: Token) -> impl Fn(Self) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Parameter(
                ParameterKind::MissingUnit {
                    colon_offset: colon_token.lexeme_offset(),
                },
            )),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing minimum value in limits
    pub fn limit_missing_min(error: Self) -> Self {
        Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Parameter(
                ParameterKind::LimitMissingMin,
            )),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing comma in limits
    pub fn limit_missing_comma(error: TokenError) -> Self {
        Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Parameter(
                ParameterKind::LimitMissingComma,
            )),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing maximum value in limits
    pub fn limit_missing_max(error: Self) -> Self {
        Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Parameter(
                ParameterKind::LimitMissingMax,
            )),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for missing values in discrete limits
    pub fn limit_missing_values(error: Self) -> Self {
        Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Parameter(
                ParameterKind::LimitMissingValues,
            )),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for an unclosed bracket
    pub fn unclosed_bracket(bracket_left_token: Token) -> impl Fn(TokenError) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::UnclosedBracket {
                bracket_left_offset: bracket_left_token.lexeme_offset(),
            }),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing expression in piecewise
    pub fn piecewise_missing_expr(brace_left_token: Token) -> impl Fn(Self) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Parameter(
                ParameterKind::PiecewiseMissingExpr {
                    brace_left_offset: brace_left_token.lexeme_offset(),
                },
            )),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing if keyword in piecewise
    pub fn piecewise_missing_if(brace_left_token: Token) -> impl Fn(TokenError) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Parameter(
                ParameterKind::PiecewiseMissingIf {
                    brace_left_offset: brace_left_token.lexeme_offset(),
                },
            )),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing if expression in piecewise
    pub fn piecewise_missing_if_expr(brace_left_token: Token) -> impl Fn(Self) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Parameter(
                ParameterKind::PiecewiseMissingIfExpr {
                    brace_left_offset: brace_left_token.lexeme_offset(),
                },
            )),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing colon in a test declaration
    pub fn test_missing_colon(test_token: Token) -> impl Fn(TokenError) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Test(TestKind::MissingColon {
                test_offset: test_token.lexeme_offset(),
            })),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing expression in a test declaration
    pub fn test_missing_expr(test_token: Token) -> impl Fn(Self) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Test(TestKind::MissingExpr {
                test_offset: test_token.lexeme_offset(),
            })),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing end of line in a test declaration
    pub fn test_missing_end_of_line(test_token: Token) -> impl Fn(TokenError) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Test(TestKind::MissingEndOfLine {
                test_offset: test_token.lexeme_offset(),
            })),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for missing inputs in a test declaration
    pub fn test_missing_inputs(brace_left_token: Token) -> impl Fn(TokenError) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Test(TestKind::MissingInputs {
                brace_left_offset: brace_left_token.lexeme_offset(),
            })),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for an unclosed brace
    pub fn unclosed_brace(brace_left_token: Token) -> impl Fn(TokenError) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::UnclosedBrace {
                brace_left_offset: brace_left_token.lexeme_offset(),
            }),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing second term in a unit expression
    pub fn unit_missing_second_term(operator: crate::ast::unit::UnitOp) -> impl Fn(Self) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Unit(UnitKind::MissingSecondTerm {
                operator,
                operator_offset: error.offset,
            })),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing exponent in a unit expression
    pub fn unit_missing_exponent(caret_token: Token) -> impl Fn(TokenError) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Unit(UnitKind::MissingExponent {
                caret_offset: caret_token.lexeme_offset(),
            })),
            offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing expression in parenthesized unit
    pub fn unit_paren_missing_expr(paren_left_token: Token) -> impl Fn(Self) -> Self {
        move |error| Self {
            kind: ParserErrorKind::Incomplete(IncompleteKind::Unit(UnitKind::ParenMissingExpr {
                paren_left_offset: paren_left_token.lexeme_offset(),
            })),
            offset: error.offset,
        }
    }
}

/// The different kinds of errors that can occur during parsing.
///
/// This enum represents all possible high-level parsing errors in the Oneil
/// language. Each variant describes a specific type of error, such as an
/// invalid declaration or an unexpected token.
///
/// # Examples
///
/// ```
/// use oneil::parser::error::ParserErrorKind;
///
/// // An error for an invalid number literal
/// let error = ParserErrorKind::InvalidNumber("123.4.5");
///
/// // An error for an invalid expression
/// let error = ParserErrorKind::ExpectExpr;
/// ```
// TODO: Some of the errors in this enum are redundant (ex.
//       `ImportKind::MissingEndOfLine` and `FromKind::MissingEndOfLine`)
//       We should find a way to combine them into a single error and offer
//       a way to add context to the error.
#[derive(Debug, Clone, PartialEq)]
pub enum ParserErrorKind {
    /// Expected an AST node but found something else
    Expect(ExpectKind),
    /// Found an incomplete input
    Incomplete(IncompleteKind),
    /// Found an unexpected token
    UnexpectedToken,
    /// A token-level error occurred
    TokenError(TokenErrorKind),
    /// A low-level nom parsing error
    NomError(nom::error::ErrorKind),
}

/// The different kinds of AST nodes that can be expected
#[derive(Debug, Clone, PartialEq)]
pub enum ExpectKind {
    /// Expected a declaration
    Decl,
    /// Expected an expression
    Expr,
    /// Expected a note
    Note,
    /// Expected a parameter
    Parameter,
    /// Expected a test
    Test,
    /// Expected a unit
    Unit,
}

/// The different kinds of incomplete input that can be found
#[derive(Debug, Clone, PartialEq)]
pub enum IncompleteKind {
    /// Found an incomplete declaration
    Decl(DeclKind),
    /// Found an incomplete expression
    Expr(ExprKind),
    /// Found an incomplete section
    Section(SectionKind),
    /// Found an incomplete parameter
    Parameter(ParameterKind),
    /// Found an incomplete test
    Test(TestKind),
    /// Found an incomplete unit
    Unit(UnitKind),
    /// Found an unclosed bracket
    UnclosedBracket {
        /// The offset of the opening bracket
        bracket_left_offset: usize,
    },
    /// Found an unclosed parenthesis
    UnclosedParen {
        /// The offset of the opening parenthesis
        paren_left_offset: usize,
    },
    /// Found an unclosed brace
    UnclosedBrace {
        /// The offset of the opening brace
        brace_left_offset: usize,
    },
    /// Found an invalid number with the given text
    InvalidNumber(String),
}

/// The different kind of incomplete declaration errors
#[derive(Debug, Clone, PartialEq)]
pub enum DeclKind {
    /// Found an incomplete `import` declaration
    Import {
        /// The offset of the `import` keyword
        import_offset: usize,
        /// The kind of import errorj
        kind: ImportKind,
    },
    /// Found an incomplete `from` declaration
    From {
        /// The offset of the `from` keyword
        from_offset: usize,
        /// The kind of from error
        kind: FromKind,
    },
    /// Found an incomplete `use` declaration
    Use {
        /// The offset of the `use` keyword
        use_offset: usize,
        /// The kind of use error
        kind: UseKind,
    },
    /// Model input is missing a value
    ModelInputMissingValue {
        /// The identifier of the model input
        identifier: String,
        /// The offset of the identifier
        identifier_offset: usize,
        /// The offset of the equals sign
        equals_offset: usize,
    },
}

/// The different kind of `import` errors
#[derive(Debug, Clone, PartialEq)]
pub enum ImportKind {
    /// Found an incomplete path
    MissingPath,
    /// Missing end of line
    MissingEndOfLine,
}

/// The different kind of `from` errors
#[derive(Debug, Clone, PartialEq)]
pub enum FromKind {
    /// Found an incomplete path
    MissingPath,
    /// Missing the `use` keyword
    MissingUse,
    /// Missing the model to use
    MissingUseModel {
        /// The offset of the `use` keyword
        use_offset: usize,
    },
    /// Missing the `as` keyword
    MissingAs,
    /// Missing the name to use the model as
    MissingAsName,
    /// Missing end of line
    MissingEndOfLine,
}

/// The different kind of `use` errors
#[derive(Debug, Clone, PartialEq)]
pub enum UseKind {
    /// Found an incomplete path
    MissingPath,
    /// Missing the `as` keyword
    MissingAs,
    /// Missing the name to use the model as
    MissingAsName,
    /// Missing end of line
    MissingEndOfLine,
}

/// The different kind of incomplete expression errors
#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    /// Found a binary operation missing a second operand
    BinaryOpMissingSecondOperand {
        /// The operator
        operator: BinaryOp,
        /// The offset of the operator
        operator_offset: usize,
    },
    /// Found a unary operation missing its operand
    UnaryOpMissingOperand {
        /// The operator
        operator: UnaryOp,
        /// The offset of the operator
        operator_offset: usize,
    },
}

/// The different kind of incomplete section errors
#[derive(Debug, Clone, PartialEq)]
pub enum SectionKind {
    /// Found an incomplete section label
    MissingLabel {
        /// The offset of the section keyword
        section_offset: usize,
    },
    /// Missing end of line
    MissingEndOfLine {
        /// The offset of the section keyword
        section_offset: usize,
    },
}

/// The different kind of incomplete parameter errors
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterKind {
    /// Found a missing identifier
    MissingIdentifier,
    /// Found a missing equals sign
    MissingEqualsSign {
        /// The offset of the identifier
        identifier_offset: usize,
        /// The offset of the identifier end
        identifier_end_offset: usize,
    },
    /// Found a missing value
    MissingValue {
        /// The offset of the identifier
        identifier_offset: usize,
        /// The offset of the identifier end
        identifier_end_offset: usize,
    },
    /// Found a missing end of line
    MissingEndOfLine {
        /// The offset of the identifier
        identifier_offset: usize,
        /// The offset of the identifier end
        identifier_end_offset: usize,
    },
    /// Found a missing unit
    MissingUnit {
        /// The offset of the colon
        colon_offset: usize,
    },
    /// Found a missing minimum value in limits
    LimitMissingMin,
    /// Found a missing comma in limits
    LimitMissingComma,
    /// Found a missing maximum value in limits
    LimitMissingMax,
    /// Found missing values in discrete limits
    LimitMissingValues,
    /// Found a missing expression in piecewise
    PiecewiseMissingExpr {
        /// The offset of the opening brace
        brace_left_offset: usize,
    },
    /// Found a missing if keyword in piecewise
    PiecewiseMissingIf {
        /// The offset of the opening brace
        brace_left_offset: usize,
    },
    /// Found a missing if expression in piecewise
    PiecewiseMissingIfExpr {
        /// The offset of the opening brace
        brace_left_offset: usize,
    },
}

/// The different kind of incomplete test errors
#[derive(Debug, Clone, PartialEq)]
pub enum TestKind {
    /// Found a missing colon in a test declaration
    MissingColon {
        /// The offset of the test keyword
        test_offset: usize,
    },
    /// Found a missing expression in a test declaration
    MissingExpr {
        /// The offset of the test keyword
        test_offset: usize,
    },
    /// Found a missing end of line in a test declaration
    MissingEndOfLine {
        /// The offset of the test keyword
        test_offset: usize,
    },
    /// Found missing inputs in a test declaration
    MissingInputs {
        /// The offset of the opening brace
        brace_left_offset: usize,
    },
}

/// The different kind of incomplete unit errors
#[derive(Debug, Clone, PartialEq)]
pub enum UnitKind {
    /// Found a missing second term in a unit expression
    MissingSecondTerm {
        /// The operator
        operator: crate::ast::unit::UnitOp,
        /// The offset of the operator
        operator_offset: usize,
    },
    /// Found a missing exponent in a unit expression
    MissingExponent {
        /// The offset of the caret
        caret_offset: usize,
    },
    /// Found a missing expression in parenthesized unit
    ParenMissingExpr {
        /// The offset of the opening parenthesis
        paren_left_offset: usize,
    },
}

impl<'a> ParseError<Span<'a>> for ParserError {
    fn from_error_kind(input: Span<'a>, kind: nom::error::ErrorKind) -> Self {
        let kind = match kind {
            // If `all_consuming` is used, we expect the parser to consume the entire input
            nom::error::ErrorKind::Eof => ParserErrorKind::UnexpectedToken,
            _ => ParserErrorKind::NomError(kind),
        };

        Self {
            kind,
            offset: input.location_offset(),
        }
    }

    fn append(_input: Span<'a>, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

impl<'a, F> FromExternalError<Span<'a>, F> for ParserError
where
    F: Fn() -> ParserError,
{
    fn from_external_error(_input: Span<'a>, _kind: nom::error::ErrorKind, f: F) -> Self {
        f()
    }
}

/// Implements conversion from TokenError to ParserError.
///
/// This allows token-level errors to be converted into parser-level errors
/// while preserving the error information.
impl From<TokenError> for ParserError {
    fn from(e: TokenError) -> Self {
        Self {
            kind: ParserErrorKind::TokenError(e.kind),
            offset: e.offset,
        }
    }
}
