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
//! Note that all constructor methods in both `ParserError` and `ParserErrorReason`
//! have been made `pub(crate)` to improve encapsulation.
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
use oneil_ast::{
    Span as AstSpan,
    expression::{BinaryOpNode, UnaryOpNode},
    span::SpanLike,
    unit::UnitOpNode,
};

use crate::{
    Span,
    token::{
        Token,
        error::{TokenError, TokenErrorKind},
    },
};

pub mod reason;
use reason::ParserErrorReason;

mod parser_trait;
pub use parser_trait::ErrorHandlingParser;

pub mod partial;
pub use partial::ErrorsWithPartialResult;

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
        let is_token_expect_error = matches!(
            self.reason,
            ParserErrorReason::TokenError(TokenErrorKind::Expect(_))
        );

        let is_parser_expect_error = matches!(self.reason, ParserErrorReason::Expect(_));

        assert!(
            is_token_expect_error || is_parser_expect_error,
            "Cannot convert a non-expect error to a parser error (attempted to convert {:?} to {:?})",
            self.reason,
            reason,
        );

        Self { reason, ..self }
    }

    /// Creates a new ParserError from a TokenError with a specific reason
    ///
    /// This is used to convert token-level errors to parser-level errors
    fn new_from_token_error(error: TokenError, reason: ParserErrorReason) -> Self {
        Self {
            reason,
            error_offset: error.offset,
        }
    }

    /// Creates a new ParserError for an expected declaration
    pub(crate) fn expect_decl(error: Self) -> Self {
        error.convert_reason(ParserErrorReason::expect_decl())
    }

    /// Creates a new ParserError for an expected expression
    pub(crate) fn expect_expr(error: Self) -> Self {
        error.convert_reason(ParserErrorReason::expect_expr())
    }

    /// Creates a new ParserError for an expected note
    pub(crate) fn expect_note(error: TokenError) -> Self {
        Self::new_from_token_error(error, ParserErrorReason::expect_note())
    }

    /// Creates a new ParserError for an expected parameter
    pub(crate) fn expect_parameter(error: TokenError) -> Self {
        Self::new_from_token_error(error, ParserErrorReason::expect_parameter())
    }

    /// Creates a new ParserError for an expected test
    pub(crate) fn expect_test(error: TokenError) -> Self {
        Self::new_from_token_error(error, ParserErrorReason::expect_test())
    }

    /// Creates a new ParserError for an expected unit
    pub(crate) fn expect_unit(error: Self) -> Self {
        error.convert_reason(ParserErrorReason::expect_unit())
    }

    /// Creates a new ParserError for a missing path in an import declaration
    pub(crate) fn import_missing_path(import_span: &impl SpanLike) -> impl Fn(TokenError) -> Self {
        move |error| {
            let import_span = AstSpan::from(import_span);
            Self::new_from_token_error(error, ParserErrorReason::import_missing_path(import_span))
        }
    }

    /// Creates a new ParserError for a missing end of line in an import declaration
    pub(crate) fn import_missing_end_of_line(
        import_path_span: &impl SpanLike,
    ) -> impl Fn(TokenError) -> Self {
        move |error| {
            let import_path_span = AstSpan::from(import_path_span);
            Self::new_from_token_error(
                error,
                ParserErrorReason::import_missing_end_of_line(import_path_span),
            )
        }
    }

    /// Creates a new ParserError for a missing path in a from declaration
    pub(crate) fn from_missing_path(from_span: &impl SpanLike) -> impl Fn(Self) -> Self {
        move |error| {
            let from_span = AstSpan::from(from_span);
            error.convert_reason(ParserErrorReason::from_missing_path(from_span))
        }
    }

    /// Creates a new ParserError for a missing use keyword in a from declaration
    pub(crate) fn from_missing_use(from_path: &impl SpanLike) -> impl Fn(TokenError) -> Self {
        move |error| {
            let from_path_span = AstSpan::from(from_path);
            Self::new_from_token_error(error, ParserErrorReason::from_missing_use(from_path_span))
        }
    }

    /// Creates a new ParserError for a missing use model in a from declaration
    pub(crate) fn from_missing_use_model(use_token: &impl SpanLike) -> impl Fn(TokenError) -> Self {
        move |error| {
            let use_span = AstSpan::from(use_token);
            Self::new_from_token_error(error, ParserErrorReason::from_missing_use_model(use_span))
        }
    }

    /// Creates a new ParserError for a missing as keyword in a from declaration
    pub(crate) fn from_missing_as(
        use_model_or_inputs: &impl SpanLike,
    ) -> impl Fn(TokenError) -> Self {
        move |error| {
            let use_model_or_inputs_span = AstSpan::from(use_model_or_inputs);
            Self::new_from_token_error(
                error,
                ParserErrorReason::from_missing_as(use_model_or_inputs_span),
            )
        }
    }

    /// Creates a new ParserError for a missing alias in a from declaration
    pub(crate) fn from_missing_alias(as_token: &impl SpanLike) -> impl Fn(TokenError) -> Self {
        move |error| {
            let as_span = AstSpan::from(as_token);
            Self::new_from_token_error(error, ParserErrorReason::from_missing_alias(as_span))
        }
    }

    /// Creates a new ParserError for a missing end of line in a from declaration
    pub(crate) fn from_missing_end_of_line(
        alias_span: &impl SpanLike,
    ) -> impl Fn(TokenError) -> Self {
        move |error| {
            let alias_span = AstSpan::from(alias_span);
            Self::new_from_token_error(
                error,
                ParserErrorReason::from_missing_end_of_line(alias_span),
            )
        }
    }

    /// Creates a new ParserError for a missing path in a use declaration
    pub(crate) fn use_missing_path(use_token: &impl SpanLike) -> impl Fn(Self) -> Self {
        move |error| {
            let use_span = AstSpan::from(use_token);
            error.convert_reason(ParserErrorReason::use_missing_path(use_span))
        }
    }

    /// Creates a new ParserError for a missing as keyword in a use declaration
    pub(crate) fn use_missing_as(
        use_path_or_inputs: &impl SpanLike,
    ) -> impl Fn(TokenError) -> Self {
        move |error| {
            let use_path_or_inputs_span = AstSpan::from(use_path_or_inputs);
            Self::new_from_token_error(
                error,
                ParserErrorReason::use_missing_as(use_path_or_inputs_span),
            )
        }
    }

    /// Creates a new ParserError for a missing alias in a use declaration
    pub(crate) fn use_missing_alias(as_token: &impl SpanLike) -> impl Fn(TokenError) -> Self {
        move |error| {
            let as_span = AstSpan::from(as_token);
            Self::new_from_token_error(error, ParserErrorReason::use_missing_alias(as_span))
        }
    }

    /// Creates a new ParserError for a missing end of line in a use declaration
    pub(crate) fn use_missing_end_of_line(
        alias_span: &impl SpanLike,
    ) -> impl Fn(TokenError) -> Self {
        move |error| {
            let alias_span = AstSpan::from(alias_span);
            Self::new_from_token_error(
                error,
                ParserErrorReason::use_missing_end_of_line(alias_span),
            )
        }
    }

    /// Creates a new ParserError for a missing value in a model input
    pub(crate) fn model_input_missing_value(equals_span: &impl SpanLike) -> impl Fn(Self) -> Self {
        move |error| {
            let equals_span = AstSpan::from(equals_span);
            error.convert_reason(ParserErrorReason::model_input_missing_value(equals_span))
        }
    }

    /// Creates a new ParserError for a missing subcomponent in a model path
    pub(crate) fn model_path_missing_subcomponent(
        dot_token: &impl SpanLike,
    ) -> impl Fn(TokenError) -> Self {
        move |error| {
            let dot_span = AstSpan::from(dot_token);
            Self::new_from_token_error(
                error,
                ParserErrorReason::model_path_missing_subcomponent(dot_span),
            )
        }
    }

    /// Creates a new ParserError for a binary operation missing its second operand
    pub(crate) fn expr_binary_op_missing_second_operand(
        operator: &BinaryOpNode,
    ) -> impl Fn(Self) -> Self {
        move |error| {
            let operator_span = AstSpan::from(operator);
            let operator = operator.node_value().clone();
            error.convert_reason(ParserErrorReason::expr_binary_op_missing_second_operand(
                operator_span,
                operator,
            ))
        }
    }

    /// Creates a new ParserError for a unary operation missing its operand
    pub(crate) fn unary_op_missing_operand(operator: &UnaryOpNode) -> impl Fn(Self) -> Self {
        move |error| {
            let operator_span = AstSpan::from(operator);
            let operator = operator.node_value().clone();
            error.convert_reason(ParserErrorReason::expr_unary_op_missing_operand(
                operator_span,
                operator,
            ))
        }
    }

    /// Creates a new ParserError for a parenthesis missing its expression
    pub(crate) fn expr_paren_missing_expression(
        paren_left_token: &impl SpanLike,
    ) -> impl Fn(Self) -> Self {
        move |error| {
            let paren_left_span = AstSpan::from(paren_left_token);
            error.convert_reason(ParserErrorReason::expr_paren_missing_expr(paren_left_span))
        }
    }

    /// Creates a new ParserError for a missing parent in a variable accessor
    pub(crate) fn expr_variable_missing_parent(
        dot_span: &impl SpanLike,
    ) -> impl Fn(TokenError) -> Self {
        move |error| {
            let dot_span = AstSpan::from(dot_span);
            Self::new_from_token_error(
                error,
                ParserErrorReason::expr_variable_missing_parent_model(dot_span),
            )
        }
    }

    /// Creates a new ParserError for a section missing a label
    pub(crate) fn section_missing_label(section_token: &Token) -> impl Fn(TokenError) -> Self {
        move |error| {
            let section_span = AstSpan::from(section_token);
            Self::new_from_token_error(
                error,
                ParserErrorReason::section_missing_label(section_span),
            )
        }
    }

    /// Creates a new ParserError for a section missing an end of line
    pub(crate) fn section_missing_end_of_line(
        section_label_span: &impl SpanLike,
    ) -> impl Fn(TokenError) -> Self {
        move |error| {
            let section_label_span = AstSpan::from(section_label_span);
            Self::new_from_token_error(
                error,
                ParserErrorReason::section_missing_end_of_line(section_label_span),
            )
        }
    }

    /// Creates a new ParserError for a missing identifier in a parameter
    pub(crate) fn parameter_missing_identifier(
        colon_token: &impl SpanLike,
    ) -> impl Fn(TokenError) -> Self {
        move |error| {
            let colon_span = AstSpan::from(colon_token);
            Self::new_from_token_error(
                error,
                ParserErrorReason::parameter_missing_identifier(colon_span),
            )
        }
    }

    /// Creates a new ParserError for a missing equals sign in a parameter
    pub(crate) fn parameter_missing_equals_sign(
        ident_span: &impl SpanLike,
    ) -> impl Fn(TokenError) -> Self {
        move |error| {
            let ident_span = AstSpan::from(ident_span);
            Self::new_from_token_error(
                error,
                ParserErrorReason::parameter_missing_equals_sign(ident_span),
            )
        }
    }

    /// Creates a new ParserError for a missing value in a parameter
    pub(crate) fn parameter_missing_value(equals_token: &impl SpanLike) -> impl Fn(Self) -> Self {
        move |error| {
            let equals_span = AstSpan::from(equals_token);
            error.convert_reason(ParserErrorReason::parameter_missing_value(equals_span))
        }
    }

    /// Creates a new ParserError for a missing end of line in a parameter
    pub(crate) fn parameter_missing_end_of_line(
        value_span: &impl SpanLike,
    ) -> impl Fn(TokenError) -> Self {
        move |error| {
            let value_span = AstSpan::from(value_span);
            Self::new_from_token_error(
                error,
                ParserErrorReason::parameter_missing_end_of_line(value_span),
            )
        }
    }

    /// Creates a new ParserError for a missing unit in a parameter
    pub(crate) fn parameter_missing_unit(colon_token: &impl SpanLike) -> impl Fn(Self) -> Self {
        move |error| {
            eprintln!("error: {:?}", error);
            let colon_span = AstSpan::from(colon_token);
            error.convert_reason(ParserErrorReason::parameter_missing_unit(colon_span))
        }
    }

    /// Creates a new ParserError for a missing minimum value in limits
    pub(crate) fn limit_missing_min(paren_left_token: &impl SpanLike) -> impl Fn(Self) -> Self {
        move |error| {
            let left_paren_span = AstSpan::from(paren_left_token);
            error.convert_reason(ParserErrorReason::limit_missing_min(left_paren_span))
        }
    }

    /// Creates a new ParserError for a missing comma in limits
    pub(crate) fn limit_missing_comma(min_span: &impl SpanLike) -> impl Fn(TokenError) -> Self {
        move |error| {
            let min_span = AstSpan::from(min_span);
            Self::new_from_token_error(error, ParserErrorReason::limit_missing_comma(min_span))
        }
    }

    /// Creates a new ParserError for a missing maximum value in limits
    pub(crate) fn limit_missing_max(comma_token: &impl SpanLike) -> impl Fn(Self) -> Self {
        move |error| {
            let comma_span = AstSpan::from(comma_token);
            error.convert_reason(ParserErrorReason::limit_missing_max(comma_span))
        }
    }

    /// Creates a new ParserError for missing values in discrete limits
    pub(crate) fn limit_missing_values(
        bracket_left_token: &impl SpanLike,
    ) -> impl Fn(Self) -> Self {
        move |error| {
            let bracket_left_span = AstSpan::from(bracket_left_token);
            error.convert_reason(ParserErrorReason::limit_missing_values(bracket_left_span))
        }
    }

    /// Creates a new ParserError for a missing expression in piecewise
    pub(crate) fn piecewise_missing_expr(
        brace_left_token: &impl SpanLike,
    ) -> impl Fn(Self) -> Self {
        move |error| {
            let brace_left_span = AstSpan::from(brace_left_token);
            error.convert_reason(ParserErrorReason::piecewise_missing_expr(brace_left_span))
        }
    }

    /// Creates a new ParserError for a missing if keyword in piecewise
    pub(crate) fn piecewise_missing_if(expr_span: &impl SpanLike) -> impl Fn(TokenError) -> Self {
        move |error| {
            let expr_span = AstSpan::from(expr_span);
            Self::new_from_token_error(error, ParserErrorReason::piecewise_missing_if(expr_span))
        }
    }

    /// Creates a new ParserError for a missing if expression in piecewise
    pub(crate) fn piecewise_missing_if_expr(if_token: &impl SpanLike) -> impl Fn(Self) -> Self {
        move |error| {
            let if_span = AstSpan::from(if_token);
            error.convert_reason(ParserErrorReason::piecewise_missing_if_expr(if_span))
        }
    }

    /// Creates a new ParserError for a missing colon in a test declaration
    pub(crate) fn test_missing_colon(
        test_kw_or_inputs_span: &impl SpanLike,
    ) -> impl Fn(TokenError) -> Self {
        move |error| {
            let test_kw_or_inputs_span = AstSpan::from(test_kw_or_inputs_span);
            Self::new_from_token_error(
                error,
                ParserErrorReason::test_missing_colon(test_kw_or_inputs_span),
            )
        }
    }

    /// Creates a new ParserError for a missing expression in a test declaration
    pub(crate) fn test_missing_expr(colon_token: &impl SpanLike) -> impl Fn(Self) -> Self {
        move |error| {
            let colon_span = AstSpan::from(colon_token);
            error.convert_reason(ParserErrorReason::test_missing_expr(colon_span))
        }
    }

    /// Creates a new ParserError for a missing end of line in a test declaration
    pub(crate) fn test_missing_end_of_line(
        expr_span: &impl SpanLike,
    ) -> impl Fn(TokenError) -> Self {
        move |error| {
            let expr_span = AstSpan::from(expr_span);
            Self::new_from_token_error(
                error,
                ParserErrorReason::test_missing_end_of_line(expr_span),
            )
        }
    }

    /// Creates a new ParserError for missing inputs in a test declaration
    pub(crate) fn test_missing_inputs(
        brace_left_token: &impl SpanLike,
    ) -> impl Fn(TokenError) -> Self {
        move |error| {
            let brace_left_span = AstSpan::from(brace_left_token);
            Self::new_from_token_error(
                error,
                ParserErrorReason::test_missing_inputs(brace_left_span),
            )
        }
    }

    /// Creates a new ParserError for a missing second term in a unit expression
    pub(crate) fn unit_missing_second_term(operator_node: &UnitOpNode) -> impl Fn(Self) -> Self {
        move |error| {
            let operator_span = AstSpan::from(operator_node);
            let operator = operator_node.node_value().clone();
            error.convert_reason(ParserErrorReason::unit_missing_second_term(
                operator_span,
                operator,
            ))
        }
    }

    /// Creates a new ParserError for a missing exponent in a unit expression
    pub(crate) fn unit_missing_exponent(
        caret_token: &impl SpanLike,
    ) -> impl Fn(TokenError) -> Self {
        move |error| {
            let caret_span = AstSpan::from(caret_token);
            Self::new_from_token_error(error, ParserErrorReason::unit_missing_exponent(caret_span))
        }
    }

    /// Creates a new ParserError for a missing expression in parenthesized unit
    pub(crate) fn unit_paren_missing_expr(
        paren_left_token: &impl SpanLike,
    ) -> impl Fn(Self) -> Self {
        move |error| {
            let paren_left_span = AstSpan::from(paren_left_token);
            error.convert_reason(ParserErrorReason::unit_paren_missing_expr(paren_left_span))
        }
    }

    /// Creates a new ParserError for an unclosed brace
    pub(crate) fn unclosed_brace(brace_left_token: &impl SpanLike) -> impl Fn(TokenError) -> Self {
        move |error| {
            let brace_left_span = AstSpan::from(brace_left_token);
            Self::new_from_token_error(error, ParserErrorReason::unclosed_brace(brace_left_span))
        }
    }

    /// Creates a new ParserError for an unclosed bracket
    pub(crate) fn unclosed_bracket(
        bracket_left_token: &impl SpanLike,
    ) -> impl Fn(TokenError) -> Self {
        move |error| {
            let bracket_left_span = AstSpan::from(bracket_left_token);
            Self::new_from_token_error(
                error,
                ParserErrorReason::unclosed_bracket(bracket_left_span),
            )
        }
    }

    /// Creates a new ParserError for an unclosed parenthesis
    pub(crate) fn unclosed_paren(paren_left_token: &impl SpanLike) -> impl Fn(TokenError) -> Self {
        move |error| {
            let paren_left_span = AstSpan::from(paren_left_token);
            Self::new_from_token_error(error, ParserErrorReason::unclosed_paren(paren_left_span))
        }
    }
}

impl<'a> ParseError<Span<'a>> for ParserError {
    fn from_error_kind(input: Span<'a>, reason: nom::error::ErrorKind) -> Self {
        let reason = match reason {
            // If `all_consuming` is used, we expect the parser to consume the entire input
            nom::error::ErrorKind::Eof => ParserErrorReason::unexpected_token(),
            _ => ParserErrorReason::nom_error(reason),
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
            reason: ParserErrorReason::token_error(e.kind),
            error_offset: e.offset,
        }
    }
}
