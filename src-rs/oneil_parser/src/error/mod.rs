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

use nom::error::ParseError;

use crate::{
    Span,
    token::{
        Token,
        error::{TokenError, TokenErrorKind},
    },
};

use oneil_ast::{
    Span as AstSpan,
    expression::{BinaryOp, UnaryOp, UnaryOpNode},
    unit::UnitOpNode,
};
use oneil_ast::{expression::BinaryOpNode, unit::UnitOp};

mod parser_trait;
pub use parser_trait::ErrorHandlingParser;

pub mod partial;
pub use partial::ErrorsWithPartialResult;

// TODO: make all constructors in this file `pub(crate)`

/// An error that occurred during parsing.
///
/// This type represents high-level parsing errors, containing both the specific
/// kind of error and the location where it occurred. It is used for errors that
/// occur during the parsing of language constructs like declarations, expressions,
/// and parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct ParserError {
    /// The location in the source where the error occurred
    pub error_offset: usize,
    /// The reason for the error
    pub reason: ParserErrorReason,
}

impl ParserError {
    /// Converts the error kind to a new kind
    ///
    /// This is used to convert a wrapped token error to a parser error
    fn convert_reason(self, reason: ParserErrorReason) -> Self {
        let is_token_error = matches!(
            self.reason,
            ParserErrorReason::TokenError(TokenErrorKind::Expect(_))
        );
        assert!(
            is_token_error,
            "Cannot convert a non-token error to a parser error (attempted to convert {:?})",
            self.reason
        );

        Self { reason, ..self }
    }

    /// Creates a new ParserError for an expected declaration
    pub fn expect_decl(error: Self) -> Self {
        Self {
            reason: ParserErrorReason::expect_decl(),
            error_offset: error.error_offset,
        }
    }

    /// Creates a new ParserError for an expected expression
    pub fn expect_expr(error: Self) -> Self {
        Self {
            reason: ParserErrorReason::expect_expr(),
            error_offset: error.error_offset,
        }
    }

    /// Creates a new ParserError for an expected note
    pub fn expect_note(error: TokenError) -> Self {
        Self {
            reason: ParserErrorReason::expect_note(),
            error_offset: error.offset,
        }
    }

    /// Creates a new ParserError for an expected parameter
    pub fn expect_parameter(error: TokenError) -> Self {
        Self {
            reason: ParserErrorReason::expect_parameter(),
            error_offset: error.offset,
        }
    }

    /// Creates a new ParserError for an expected test
    pub fn expect_test(error: TokenError) -> Self {
        Self {
            reason: ParserErrorReason::expect_test(),
            error_offset: error.offset,
        }
    }

    /// Creates a new ParserError for an expected unit
    pub fn expect_unit(error: Self) -> Self {
        Self {
            reason: ParserErrorReason::expect_unit(),
            error_offset: error.error_offset,
        }
    }

    /// Creates a new ParserError for a missing path in an import declaration
    pub fn import_missing_path(import_token: &Token) -> impl Fn(TokenError) -> Self {
        move |error| {
            let import_span = AstSpan::from(import_token);
            Self {
                reason: ParserErrorReason::import_missing_path(import_span),
                error_offset: error.offset,
            }
        }
    }

    /// Creates a new ParserError for a missing end of line in an import declaration
    pub fn import_missing_end_of_line(import_path_token: &Token) -> impl Fn(TokenError) -> Self {
        todo!();
        println!();
        move |error| {
            let import_path_span = AstSpan::from(import_path_token);
            Self {
                reason: ParserErrorReason::import_missing_end_of_line(import_path_span),
                error_offset: error.offset,
            }
        }
    }

    /// Creates a new ParserError for a missing path in a from declaration
    pub fn from_missing_path(from_token: &Token) -> impl Fn(Self) -> Self {
        todo!();
        println!();
        move |error| {
            let from_span = AstSpan::from(from_token);
            error.convert_reason(ParserErrorReason::from_missing_path(from_span))
        }
    }

    /// Creates a new ParserError for a missing use keyword in a from declaration
    pub fn from_missing_use(from_path_token: &Token) -> impl Fn(TokenError) -> Self {
        todo!();
        println!();
        move |error| {
            let from_path_span = AstSpan::from(from_path_token);
            Self {
                reason: ParserErrorReason::from_missing_use(from_path_span),
                error_offset: error.offset,
            }
        }
    }

    /// Creates a new ParserError for a missing use model in a from declaration
    pub fn from_missing_use_model(use_token: &Token) -> impl Fn(TokenError) -> Self {
        todo!();
        println!();
        move |error| {
            let use_span = AstSpan::from(use_token);
            Self {
                reason: ParserErrorReason::from_missing_use_model(use_span),
                error_offset: error.offset,
            }
        }
    }

    /// Creates a new ParserError for a missing as keyword in a from declaration
    pub fn from_missing_as(use_model_token: &Token) -> impl Fn(TokenError) -> Self {
        todo!();
        println!();
        move |error| {
            let use_model_span = AstSpan::from(use_model_token);
            Self {
                reason: ParserErrorReason::from_missing_as(use_model_span),
                error_offset: error.offset,
            }
        }
    }

    /// Creates a new ParserError for a missing alias in a from declaration
    pub fn from_missing_alias(as_token: &Token) -> impl Fn(TokenError) -> Self {
        todo!();
        println!();
        move |error| {
            let as_span = AstSpan::from(as_token);
            Self {
                reason: ParserErrorReason::from_missing_alias(as_span),
                error_offset: error.offset,
            }
        }
    }

    /// Creates a new ParserError for a missing end of line in a from declaration
    pub fn from_missing_end_of_line(alias_token: &Token) -> impl Fn(TokenError) -> Self {
        todo!();
        println!();
        move |error| {
            let alias_span = AstSpan::from(alias_token);
            Self {
                reason: ParserErrorReason::from_missing_end_of_line(alias_span),
                error_offset: error.offset,
            }
        }
    }

    /// Creates a new ParserError for a missing path in a use declaration
    pub fn use_missing_path(use_token: &Token) -> impl Fn(Self) -> Self {
        todo!();
        println!();
        move |error| {
            let use_span = AstSpan::from(use_token);
            Self {
                reason: ParserErrorReason::use_missing_path(use_span),
                error_offset: error.error_offset,
            }
        }
    }

    /// Creates a new ParserError for a missing as keyword in a use declaration
    pub fn use_missing_as(use_path_token: &Token) -> impl Fn(TokenError) -> Self {
        todo!();
        println!();
        move |error| {
            let use_path_span = AstSpan::from(use_path_token);
            Self {
                reason: ParserErrorReason::use_missing_as(use_path_span),
                error_offset: error.offset,
            }
        }
    }

    /// Creates a new ParserError for a missing alias in a use declaration
    pub fn use_missing_alias(as_token: &Token) -> impl Fn(TokenError) -> Self {
        todo!();
        println!();
        move |error| {
            let as_span = AstSpan::from(as_token);
            Self {
                reason: ParserErrorReason::use_missing_alias(as_span),
                error_offset: error.offset,
            }
        }
    }

    /// Creates a new ParserError for a missing end of line in a use declaration
    pub fn use_missing_end_of_line(alias_token: &Token) -> impl Fn(TokenError) -> Self {
        todo!();
        println!();
        move |error| {
            let alias_span = AstSpan::from(alias_token);
            Self {
                reason: ParserErrorReason::use_missing_end_of_line(alias_span),
                error_offset: error.offset,
            }
        }
    }

    /// Creates a new ParserError for a missing value in a model input
    pub fn model_input_missing_value(equals_token: &Token) -> impl Fn(Self) -> Self {
        todo!();
        println!();
        move |error| {
            let equals_span = AstSpan::from(equals_token);
            Self {
                reason: ParserErrorReason::model_input_missing_value(equals_span),
                error_offset: error.error_offset,
            }
        }
    }

    /// Creates a new ParserError for a missing subcomponent in a model path
    pub fn model_path_missing_subcomponent(dot_token: &Token) -> impl Fn(TokenError) -> Self {
        todo!();
        println!();
        move |error| {
            let dot_span = AstSpan::from(dot_token);
            Self {
                reason: ParserErrorReason::model_path_missing_subcomponent(dot_span),
                error_offset: error.offset,
            }
        }
    }

    /// Creates a new ParserError for a binary operation missing its second operand
    pub fn binary_op_missing_second_operand(operator: &BinaryOpNode) -> impl Fn(Self) -> Self {
        todo!();
        println!();
        move |error| {
            let operator_span = AstSpan::from(operator);
            let operator = operator.node_value().clone();
            Self {
                reason: ParserErrorReason::expr_binary_op_missing_second_operand(
                    operator_span,
                    operator,
                ),
                error_offset: error.error_offset,
            }
        }
    }

    /// Creates a new ParserError for a unary operation missing its operand
    pub fn unary_op_missing_operand(operator: &UnaryOpNode) -> impl Fn(Self) -> Self {
        todo!();
        println!();
        move |error| {
            let operator_span = AstSpan::from(operator);
            let operator = operator.node_value().clone();
            Self {
                reason: ParserErrorReason::expr_unary_op_missing_operand(operator_span, operator),
                error_offset: error.error_offset,
            }
        }
    }

    /// Creates a new ParserError for a parenthesis missing its expression
    pub fn paren_missing_expression(paren_left_token: &Token) -> impl Fn(Self) -> Self {
        move |error| {
            let paren_left_span = AstSpan::from(paren_left_token);
            Self {
                reason: ParserErrorReason::expr_paren_missing_expr(paren_left_span),
                error_offset: error.error_offset,
            }
        }
    }

    /// Creates a new ParserError for an unclosed parenthesis
    pub fn unclosed_paren(paren_left_token: &Token) -> impl Fn(TokenError) -> Self {
        move |error| {
            let paren_left_span = AstSpan::from(paren_left_token);
            Self {
                reason: ParserErrorReason::unclosed_paren(paren_left_span),
                error_offset: error.offset,
            }
        }
    }

    /// Creates a new ParserError for a section missing a label
    pub fn section_missing_label(section_token: &Token) -> impl Fn(TokenError) -> Self {
        move |error| {
            let section_span = AstSpan::from(section_token);
            Self {
                reason: ParserErrorReason::section_missing_label(section_span),
                error_offset: error.offset,
            }
        }
    }

    /// Creates a new ParserError for a section missing an end of line
    pub fn section_missing_end_of_line(section_label_token: &Token) -> impl Fn(TokenError) -> Self {
        todo!();
        println!();
        move |error| {
            let section_label_span = AstSpan::from(section_label_token);
            Self {
                reason: ParserErrorReason::section_missing_end_of_line(section_label_span),
                error_offset: error.offset,
            }
        }
    }

    /// Creates a new ParserError for a missing identifier in a parameter
    pub fn parameter_missing_identifier(colon_token: &Token) -> impl Fn(TokenError) -> Self {
        todo!();
        println!();
        move |error| {
            let colon_span = AstSpan::from(colon_token);
            Self {
                reason: ParserErrorReason::parameter_missing_identifier(colon_span),
                error_offset: error.offset,
            }
        }
    }

    /// Creates a new ParserError for a missing equals sign in a parameter
    pub fn parameter_missing_equals_sign(
        ident_or_limit_span: AstSpan,
    ) -> impl Fn(TokenError) -> Self {
        todo!();
        println!();
        move |error| Self {
            reason: ParserErrorReason::parameter_missing_equals_sign(ident_or_limit_span),
            error_offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing value in a parameter
    pub fn parameter_missing_value(equals_token: &Token) -> impl Fn(Self) -> Self {
        todo!();
        println!();
        move |error| {
            let equals_span = AstSpan::from(equals_token);
            Self {
                reason: ParserErrorReason::parameter_missing_value(equals_span),
                error_offset: error.error_offset,
            }
        }
    }

    /// Creates a new ParserError for a missing end of line in a parameter
    pub fn parameter_missing_end_of_line(value_span: AstSpan) -> impl Fn(TokenError) -> Self {
        todo!();
        println!();
        move |error| Self {
            reason: ParserErrorReason::parameter_missing_end_of_line(value_span),
            error_offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing unit in a parameter
    pub fn parameter_missing_unit(colon_token: &Token) -> impl Fn(Self) -> Self {
        todo!();
        println!();
        move |error| {
            let colon_span = AstSpan::from(colon_token);
            Self {
                reason: ParserErrorReason::parameter_missing_unit(colon_span),
                error_offset: error.error_offset,
            }
        }
    }

    /// Creates a new ParserError for a missing minimum value in limits
    pub fn limit_missing_min(left_paren_token: &Token) -> impl Fn(Self) -> Self {
        todo!();
        println!();
        move |error| {
            let left_paren_span = AstSpan::from(left_paren_token);
            Self {
                reason: ParserErrorReason::limit_missing_min(left_paren_span),
                error_offset: error.error_offset,
            }
        }
    }

    /// Creates a new ParserError for a missing comma in limits
    pub fn limit_missing_comma(min_span: AstSpan) -> impl Fn(TokenError) -> Self {
        todo!();
        println!();
        move |error| Self {
            reason: ParserErrorReason::limit_missing_comma(min_span),
            error_offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing maximum value in limits
    pub fn limit_missing_max(comma_token: &Token) -> impl Fn(TokenError) -> Self {
        todo!();
        println!();
        move |error| {
            let comma_span = AstSpan::from(comma_token);
            Self {
                reason: ParserErrorReason::limit_missing_max(comma_span),
                error_offset: error.offset,
            }
        }
    }

    /// Creates a new ParserError for missing values in discrete limits
    pub fn limit_missing_values(left_bracket_token: &Token) -> impl Fn(TokenError) -> Self {
        todo!();
        println!();
        move |error| {
            let left_bracket_span = AstSpan::from(left_bracket_token);
            Self {
                reason: ParserErrorReason::limit_missing_values(left_bracket_span),
                error_offset: error.offset,
            }
        }
    }

    /// Creates a new ParserError for an unclosed bracket
    pub fn unclosed_bracket(bracket_left_token: &Token) -> impl Fn(TokenError) -> Self {
        move |error| {
            let bracket_left_span = AstSpan::from(bracket_left_token);
            Self {
                reason: ParserErrorReason::unclosed_bracket(bracket_left_span),
                error_offset: error.offset,
            }
        }
    }

    /// Creates a new ParserError for a missing expression in piecewise
    pub fn piecewise_missing_expr(brace_left_token: &Token) -> impl Fn(Self) -> Self {
        // TODO: make sure that we use the convert function above rather than creating a new one
        move |error| {
            let brace_left_span = AstSpan::from(brace_left_token);
            Self {
                reason: ParserErrorReason::piecewise_missing_expr(brace_left_span),
                error_offset: error.error_offset,
            }
        }
    }

    /// Creates a new ParserError for a missing if keyword in piecewise
    pub fn piecewise_missing_if(expr_span: AstSpan) -> impl Fn(TokenError) -> Self {
        todo!();
        println!();
        move |error| Self {
            reason: ParserErrorReason::piecewise_missing_if(expr_span),
            error_offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing if expression in piecewise
    pub fn piecewise_missing_if_expr(if_token: &Token) -> impl Fn(TokenError) -> Self {
        todo!();
        println!();
        move |error| {
            let if_span = AstSpan::from(if_token);
            Self {
                reason: ParserErrorReason::piecewise_missing_if_expr(if_span),
                error_offset: error.offset,
            }
        }
    }

    /// Creates a new ParserError for a missing colon in a test declaration
    pub fn test_missing_colon(test_kw_or_inputs_span: AstSpan) -> impl Fn(TokenError) -> Self {
        todo!();
        println!();
        move |error| Self {
            reason: ParserErrorReason::test_missing_colon(test_kw_or_inputs_span),
            error_offset: error.offset,
        }
    }

    /// Creates a new ParserError for a missing expression in a test declaration
    pub fn test_missing_expr(colon_token: &Token) -> impl Fn(TokenError) -> Self {
        todo!();
        println!();
        move |error| {
            let colon_span = AstSpan::from(colon_token);
            Self {
                reason: ParserErrorReason::test_missing_expr(colon_span),
                error_offset: error.offset,
            }
        }
    }

    /// Creates a new ParserError for a missing end of line in a test declaration
    pub fn test_missing_end_of_line(expr_span: AstSpan) -> impl Fn(TokenError) -> Self {
        todo!();
        println!();
        move |error| Self {
            reason: ParserErrorReason::test_missing_end_of_line(expr_span),
            error_offset: error.offset,
        }
    }

    /// Creates a new ParserError for missing inputs in a test declaration
    pub fn test_missing_inputs(brace_left_token: &Token) -> impl Fn(TokenError) -> Self {
        todo!();
        println!();
        move |error| {
            let brace_left_span = AstSpan::from(brace_left_token);
            Self {
                reason: ParserErrorReason::test_missing_inputs(brace_left_span),
                error_offset: error.offset,
            }
        }
    }

    /// Creates a new ParserError for an unclosed brace
    pub fn unclosed_brace(brace_left_token: &Token) -> impl Fn(TokenError) -> Self {
        todo!();
        println!();
        move |error| {
            let brace_left_span = AstSpan::from(brace_left_token);
            Self {
                reason: ParserErrorReason::unclosed_brace(brace_left_span),
                error_offset: error.offset,
            }
        }
    }

    /// Creates a new ParserError for a missing second term in a unit expression
    pub fn unit_missing_second_term(operator_node: &UnitOpNode) -> impl Fn(Self) -> Self {
        todo!();
        println!();
        move |error| {
            let operator_span = AstSpan::from(operator_node);
            let operator = operator_node.node_value().clone();
            Self {
                reason: ParserErrorReason::unit_missing_second_term(operator_span, operator),
                error_offset: error.error_offset,
            }
        }
    }

    /// Creates a new ParserError for a missing exponent in a unit expression
    pub fn unit_missing_exponent(caret_token: &Token) -> impl Fn(TokenError) -> Self {
        todo!();
        println!();
        move |error| {
            let caret_span = AstSpan::from(caret_token);
            Self {
                reason: ParserErrorReason::unit_missing_exponent(caret_span),
                error_offset: error.offset,
            }
        }
    }

    /// Creates a new ParserError for a missing expression in parenthesized unit
    pub fn unit_paren_missing_expr(paren_left_token: &Token) -> impl Fn(Self) -> Self {
        todo!();
        println!();
        move |error| {
            let paren_left_span = AstSpan::from(paren_left_token);
            Self {
                reason: ParserErrorReason::unit_paren_missing_expr(paren_left_span),
                error_offset: error.error_offset,
            }
        }
    }

    /// Creates a new ParserError for a missing parent in a variable accessor
    pub fn variable_missing_parent(dot_token: &Token) -> impl Fn(TokenError) -> Self {
        todo!();
        println!();
        move |error| {
            let dot_span = AstSpan::from(dot_token);
            Self {
                reason: ParserErrorReason::expr_variable_missing_parent_model(dot_span),
                error_offset: error.offset,
            }
        }
    }
}

/// The different kinds of errors that can occur during parsing.
///
/// This enum represents all possible high-level parsing errors in the Oneil
/// language. Each variant describes a specific type of error, such as an
/// invalid declaration or an unexpected token.
#[derive(Debug, Clone, PartialEq)]
pub enum ParserErrorReason {
    /// Expected an AST node but found something else
    Expect(ExpectKind),
    /// Found an incomplete input
    Incomplete {
        cause: AstSpan,
        kind: IncompleteKind,
    },
    /// Found an unexpected token
    UnexpectedToken,
    /// A token-level error occurred
    TokenError(TokenErrorKind),
    /// A low-level nom parsing error
    NomError(nom::error::ErrorKind),
}

impl ParserErrorReason {
    pub fn expect_decl() -> Self {
        Self::Expect(ExpectKind::Decl)
    }

    pub fn expect_expr() -> Self {
        Self::Expect(ExpectKind::Expr)
    }

    pub fn expect_note() -> Self {
        Self::Expect(ExpectKind::Note)
    }

    pub fn expect_parameter() -> Self {
        Self::Expect(ExpectKind::Parameter)
    }

    pub fn expect_test() -> Self {
        Self::Expect(ExpectKind::Test)
    }

    pub fn expect_unit() -> Self {
        Self::Expect(ExpectKind::Unit)
    }

    pub fn import_missing_path(import_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: import_span,
            kind: IncompleteKind::Decl(DeclKind::Import(ImportKind::MissingPath)),
        }
    }

    pub fn import_missing_end_of_line(import_path_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: import_path_span,
            kind: IncompleteKind::Decl(DeclKind::Import(ImportKind::MissingEndOfLine)),
        }
    }

    pub fn from_missing_path(from_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: from_span,
            kind: IncompleteKind::Decl(DeclKind::From(FromKind::MissingPath)),
        }
    }

    pub fn from_missing_use(from_path_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: from_path_span,
            kind: IncompleteKind::Decl(DeclKind::From(FromKind::MissingUse)),
        }
    }

    pub fn from_missing_use_model(use_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: use_span,
            kind: IncompleteKind::Decl(DeclKind::From(FromKind::MissingUseModel)),
        }
    }

    pub fn from_missing_as(use_model_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: use_model_span,
            kind: IncompleteKind::Decl(DeclKind::From(FromKind::MissingAs)),
        }
    }

    pub fn from_missing_alias(as_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: as_span,
            kind: IncompleteKind::Decl(DeclKind::From(FromKind::MissingAlias)),
        }
    }

    pub fn from_missing_end_of_line(alias_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: alias_span,
            kind: IncompleteKind::Decl(DeclKind::From(FromKind::MissingEndOfLine)),
        }
    }

    pub fn use_missing_path(use_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: use_span,
            kind: IncompleteKind::Decl(DeclKind::Use(UseKind::MissingPath)),
        }
    }

    pub fn use_missing_as(use_path_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: use_path_span,
            kind: IncompleteKind::Decl(DeclKind::Use(UseKind::MissingAs)),
        }
    }

    pub fn use_missing_alias(as_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: as_span,
            kind: IncompleteKind::Decl(DeclKind::Use(UseKind::MissingAlias)),
        }
    }

    pub fn use_missing_end_of_line(alias_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: alias_span,
            kind: IncompleteKind::Decl(DeclKind::Use(UseKind::MissingEndOfLine)),
        }
    }

    pub fn model_input_missing_value(equals_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: equals_span,
            kind: IncompleteKind::Decl(DeclKind::ModelInputMissingValue),
        }
    }

    pub fn model_path_missing_subcomponent(dot_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: dot_span,
            kind: IncompleteKind::Decl(DeclKind::ModelPathMissingSubcomponent),
        }
    }

    pub fn expr_binary_op_missing_second_operand(
        operator_span: AstSpan,
        operator: BinaryOp,
    ) -> Self {
        Self::Incomplete {
            cause: operator_span,
            kind: IncompleteKind::Expr(ExprKind::BinaryOpMissingSecondOperand { operator }),
        }
    }

    pub fn expr_paren_missing_expr(paren_left_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: paren_left_span,
            kind: IncompleteKind::Expr(ExprKind::ParenMissingExpr),
        }
    }

    pub fn expr_unary_op_missing_operand(operator_span: AstSpan, operator: UnaryOp) -> Self {
        Self::Incomplete {
            cause: operator_span,
            kind: IncompleteKind::Expr(ExprKind::UnaryOpMissingOperand { operator }),
        }
    }

    pub fn expr_variable_missing_parent_model(dot_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: dot_span,
            kind: IncompleteKind::Expr(ExprKind::VariableMissingParentModel),
        }
    }

    pub fn section_missing_label(section_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: section_span,
            kind: IncompleteKind::Section(SectionKind::MissingLabel),
        }
    }

    pub fn section_missing_end_of_line(section_label_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: section_label_span,
            kind: IncompleteKind::Section(SectionKind::MissingEndOfLine),
        }
    }

    pub fn parameter_missing_identifier(colon_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: colon_span,
            kind: IncompleteKind::Parameter(ParameterKind::MissingIdentifier),
        }
    }

    pub fn parameter_missing_equals_sign(ident_or_limit_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: ident_or_limit_span,
            kind: IncompleteKind::Parameter(ParameterKind::MissingEqualsSign),
        }
    }

    pub fn parameter_missing_value(equals_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: equals_span,
            kind: IncompleteKind::Parameter(ParameterKind::MissingValue),
        }
    }

    pub fn parameter_missing_end_of_line(value_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: value_span,
            kind: IncompleteKind::Parameter(ParameterKind::MissingEndOfLine),
        }
    }

    pub fn parameter_missing_unit(colon_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: colon_span,
            kind: IncompleteKind::Parameter(ParameterKind::MissingUnit),
        }
    }

    pub fn limit_missing_min(left_paren_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: left_paren_span,
            kind: IncompleteKind::Parameter(ParameterKind::LimitMissingMin),
        }
    }

    pub fn limit_missing_comma(min_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: min_span,
            kind: IncompleteKind::Parameter(ParameterKind::LimitMissingComma),
        }
    }

    pub fn limit_missing_max(comma_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: comma_span,
            kind: IncompleteKind::Parameter(ParameterKind::LimitMissingMax),
        }
    }

    pub fn limit_missing_values(left_bracket_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: left_bracket_span,
            kind: IncompleteKind::Parameter(ParameterKind::LimitMissingValues),
        }
    }

    pub fn piecewise_missing_expr(brace_left_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: brace_left_span,
            kind: IncompleteKind::Parameter(ParameterKind::PiecewiseMissingExpr),
        }
    }

    pub fn piecewise_missing_if(expr_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: expr_span,
            kind: IncompleteKind::Parameter(ParameterKind::PiecewiseMissingIf),
        }
    }

    pub fn piecewise_missing_if_expr(if_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: if_span,
            kind: IncompleteKind::Parameter(ParameterKind::PiecewiseMissingIfExpr),
        }
    }

    pub fn test_missing_colon(test_kw_or_inputs_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: test_kw_or_inputs_span,
            kind: IncompleteKind::Test(TestKind::MissingColon),
        }
    }

    pub fn test_missing_expr(colon_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: colon_span,
            kind: IncompleteKind::Test(TestKind::MissingExpr),
        }
    }

    pub fn test_missing_end_of_line(expr_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: expr_span,
            kind: IncompleteKind::Test(TestKind::MissingEndOfLine),
        }
    }

    pub fn test_missing_inputs(brace_left_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: brace_left_span,
            kind: IncompleteKind::Test(TestKind::MissingInputs),
        }
    }

    pub fn unit_missing_second_term(operator_span: AstSpan, operator: UnitOp) -> Self {
        Self::Incomplete {
            cause: operator_span,
            kind: IncompleteKind::Unit(UnitKind::MissingSecondTerm { operator }),
        }
    }

    pub fn unit_missing_exponent(caret_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: caret_span,
            kind: IncompleteKind::Unit(UnitKind::MissingExponent),
        }
    }

    pub fn unit_paren_missing_expr(paren_left_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: paren_left_span,
            kind: IncompleteKind::Unit(UnitKind::ParenMissingExpr),
        }
    }

    pub fn unclosed_brace(brace_left_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: brace_left_span,
            kind: IncompleteKind::UnclosedBrace,
        }
    }

    pub fn unclosed_bracket(bracket_left_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: bracket_left_span,
            kind: IncompleteKind::UnclosedBracket,
        }
    }

    pub fn unclosed_paren(paren_left_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: paren_left_span,
            kind: IncompleteKind::UnclosedParen,
        }
    }

    pub fn unexpected_token() -> Self {
        Self::UnexpectedToken
    }

    pub fn token_error(kind: TokenErrorKind) -> Self {
        Self::TokenError(kind)
    }

    pub fn nom_error(kind: nom::error::ErrorKind) -> Self {
        Self::NomError(kind)
    }
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
    /// Found an unclosed brace
    UnclosedBrace,
    /// Found an unclosed bracket
    UnclosedBracket,
    /// Found an unclosed parenthesis
    UnclosedParen,
}

/// The different kind of incomplete declaration errors
#[derive(Debug, Clone, PartialEq)]
pub enum DeclKind {
    /// Found an incomplete `import` declaration
    Import(
        /// The kind of import error
        ImportKind,
    ),
    /// Found an incomplete `from` declaration
    From(
        /// The kind of from error
        FromKind,
    ),
    /// Found an incomplete `use` declaration
    Use(
        /// The kind of use error
        UseKind,
    ),
    /// Model input is missing a value
    ModelInputMissingValue,
    /// Found an incomplete model path
    ModelPathMissingSubcomponent,
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
    MissingUseModel,
    /// Missing the `as` keyword
    MissingAs,
    /// Missing the model alias
    MissingAlias,
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
    /// Missing the model alias
    MissingAlias,
    /// Missing end of line
    MissingEndOfLine,
}

/// The different kind of incomplete expression errors
#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    /// Found a binary operation missing a second operand
    BinaryOpMissingSecondOperand {
        /// The operator value
        operator: BinaryOp,
    },
    /// Found a missing expression in parenthesized expression
    ParenMissingExpr,
    /// Found a unary operation missing its operand
    UnaryOpMissingOperand {
        /// The operator value
        operator: UnaryOp,
    },
    /// Found a missing parent model in a variable accessor
    VariableMissingParentModel,
}

/// The different kind of incomplete section errors
#[derive(Debug, Clone, PartialEq)]
pub enum SectionKind {
    /// Found an incomplete section label
    MissingLabel,
    /// Missing end of line
    MissingEndOfLine,
}

/// The different kind of incomplete parameter errors
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterKind {
    /// Found a missing identifier
    MissingIdentifier,
    /// Found a missing equals sign
    MissingEqualsSign,
    /// Found a missing value
    MissingValue,
    /// Found a missing end of line
    MissingEndOfLine,
    /// Found a missing unit
    MissingUnit,
    /// Found a missing minimum value in limits
    LimitMissingMin,
    /// Found a missing comma in limits
    LimitMissingComma,
    /// Found a missing maximum value in limits
    LimitMissingMax,
    /// Found missing values in discrete limits
    LimitMissingValues,
    /// Found a missing expression in piecewise
    PiecewiseMissingExpr,
    /// Found a missing if keyword in piecewise
    PiecewiseMissingIf,
    /// Found a missing if expression in piecewise
    PiecewiseMissingIfExpr,
}

/// The different kind of incomplete test errors
#[derive(Debug, Clone, PartialEq)]
pub enum TestKind {
    /// Found a missing colon in a test declaration
    MissingColon,
    /// Found a missing expression in a test declaration
    MissingExpr,
    /// Found a missing end of line in a test declaration
    MissingEndOfLine,
    /// Found missing inputs in a test declaration
    MissingInputs,
}

/// The different kind of incomplete unit errors
#[derive(Debug, Clone, PartialEq)]
pub enum UnitKind {
    /// Found a missing second term in a unit expression
    MissingSecondTerm {
        /// The operator value
        operator: UnitOp,
    },
    /// Found a missing exponent in a unit expression
    MissingExponent,
    /// Found a missing expression in parenthesized unit
    ParenMissingExpr,
}

impl<'a> ParseError<Span<'a>> for ParserError {
    fn from_error_kind(input: Span<'a>, reason: nom::error::ErrorKind) -> Self {
        let reason = match reason {
            // If `all_consuming` is used, we expect the parser to consume the entire input
            nom::error::ErrorKind::Eof => ParserErrorReason::UnexpectedToken,
            _ => ParserErrorReason::NomError(reason),
        };

        Self {
            reason,
            error_offset: input.location_offset(),
        }
    }

    fn append(_input: Span<'a>, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

/// Implements conversion from TokenError to ParserError.
///
/// This allows token-level errors to be converted into parser-level errors
/// while preserving the error information.
impl From<TokenError> for ParserError {
    fn from(e: TokenError) -> Self {
        Self {
            reason: ParserErrorReason::TokenError(e.kind),
            error_offset: e.offset,
        }
    }
}
