//! Error handling for the Oneil language parser.

use std::fmt;

use oneil_ast::{BinaryOpNode, ComparisonOpNode, UnaryOpNode, UnitOpNode};
use oneil_shared::{
    error::{AsOneilError, Context, ErrorLocation},
    span::Span,
};

use crate::{
    InputSpan,
    token::error::{TokenError, TokenErrorKind},
};

mod context;

mod display;

pub mod reason;
use reason::ParserErrorReason;

mod parser_trait;
pub use parser_trait::ErrorHandlingParser;

pub mod partial;
pub use partial::ErrorsWithPartialResult;

/// An error that occurred during parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    /// Creates a new `ParserError` from a `TokenError` with a specific reason
    ///
    /// This is used to convert token-level errors to parser-level errors
    #[must_use]
    const fn new_from_token_error(error: TokenError, reason: ParserErrorReason) -> Self {
        Self {
            reason,
            error_offset: error.offset,
        }
    }

    /// Creates a new `ParserError` for an expected declaration
    pub(crate) fn expect_decl(error: Self) -> Self {
        error.convert_reason(ParserErrorReason::expect_decl())
    }

    /// Creates a new `ParserError` for an expected expression
    pub(crate) fn expect_expr(error: Self) -> Self {
        error.convert_reason(ParserErrorReason::expect_expr())
    }

    /// Creates a new `ParserError` for an expected note
    pub(crate) const fn expect_note(error: TokenError) -> Self {
        Self::new_from_token_error(error, ParserErrorReason::expect_note())
    }

    /// Creates a new `ParserError` for an expected parameter
    pub(crate) const fn expect_parameter(error: TokenError) -> Self {
        Self::new_from_token_error(error, ParserErrorReason::expect_parameter())
    }

    /// Creates a new `ParserError` for an expected test
    pub(crate) const fn expect_test(error: TokenError) -> Self {
        Self::new_from_token_error(error, ParserErrorReason::expect_test())
    }

    /// Creates a new `ParserError` for an expected unit
    pub(crate) fn expect_unit(error: Self) -> Self {
        error.convert_reason(ParserErrorReason::expect_unit())
    }

    /// Creates a new `ParserError` for a missing path in an import declaration
    pub(crate) fn import_missing_path(import_span: Span) -> impl Fn(TokenError) -> Self {
        move |error| {
            Self::new_from_token_error(error, ParserErrorReason::import_missing_path(import_span))
        }
    }

    /// Creates a new `ParserError` for a missing end of line in an import declaration
    pub(crate) fn import_missing_end_of_line(
        import_path_span: Span,
    ) -> impl Fn(TokenError) -> Self {
        move |error| {
            Self::new_from_token_error(
                error,
                ParserErrorReason::import_missing_end_of_line(import_path_span),
            )
        }
    }

    /// Creates a new `ParserError` for a missing path in a use declaration
    pub(crate) fn use_missing_model_info(use_token: Span) -> impl Fn(Self) -> Self {
        move |error| error.convert_reason(ParserErrorReason::use_missing_model_info(use_token))
    }

    /// Creates a new `ParserError` for a missing alias in a use declaration
    pub(crate) fn as_missing_alias(as_token: Span) -> impl Fn(TokenError) -> Self {
        move |error| {
            Self::new_from_token_error(error, ParserErrorReason::as_missing_alias(as_token))
        }
    }

    /// Creates a new `ParserError` for a missing end of line in a use declaration
    pub(crate) fn use_missing_end_of_line(alias_span: Span) -> impl Fn(TokenError) -> Self {
        move |error| {
            Self::new_from_token_error(
                error,
                ParserErrorReason::use_missing_end_of_line(alias_span),
            )
        }
    }

    /// Creates a new `ParserError` for a missing subcomponent in a model path
    pub(crate) fn model_path_missing_subcomponent(dot_token: Span) -> impl Fn(TokenError) -> Self {
        move |error| {
            Self::new_from_token_error(
                error,
                ParserErrorReason::model_path_missing_subcomponent(dot_token),
            )
        }
    }

    /// Creates a new `ParserError` for a comparison operation missing its second operand
    pub(crate) fn expr_comparison_op_missing_second_operand(
        operator: &ComparisonOpNode,
    ) -> impl Fn(Self) -> Self {
        move |error| {
            let operator_span = operator.span();
            let operator = **operator;
            error.convert_reason(
                ParserErrorReason::expr_comparison_op_missing_second_operand(
                    operator_span,
                    operator,
                ),
            )
        }
    }

    /// Creates a new `ParserError` for a binary operation missing its second operand
    pub(crate) fn expr_binary_op_missing_second_operand(
        operator: &BinaryOpNode,
    ) -> impl Fn(Self) -> Self {
        move |error| {
            let operator_span = operator.span();
            let operator = **operator;
            error.convert_reason(ParserErrorReason::expr_binary_op_missing_second_operand(
                operator_span,
                operator,
            ))
        }
    }

    /// Creates a new `ParserError` for a unary operation missing its operand
    pub(crate) fn unary_op_missing_operand(operator: &UnaryOpNode) -> impl Fn(Self) -> Self {
        move |error| {
            let operator_span = operator.span();
            let operator = **operator;
            error.convert_reason(ParserErrorReason::expr_unary_op_missing_operand(
                operator_span,
                operator,
            ))
        }
    }

    /// Creates a new `ParserError` for a parenthesis missing its expression
    pub(crate) fn expr_paren_missing_expression(paren_left_token: Span) -> impl Fn(Self) -> Self {
        move |error| {
            error.convert_reason(ParserErrorReason::expr_paren_missing_expr(paren_left_token))
        }
    }

    /// Creates a new `ParserError` for a missing parent in a variable accessor
    pub(crate) fn expr_variable_missing_reference_model(
        dot_token: Span,
    ) -> impl Fn(TokenError) -> Self {
        move |error| {
            Self::new_from_token_error(
                error,
                ParserErrorReason::expr_variable_missing_reference_model(dot_token),
            )
        }
    }

    /// Creates a new `ParserError` for a section missing a label
    pub(crate) fn section_missing_label(section_token: Span) -> impl Fn(TokenError) -> Self {
        move |error| {
            Self::new_from_token_error(
                error,
                ParserErrorReason::section_missing_label(section_token),
            )
        }
    }

    /// Creates a new `ParserError` for a section missing an end of line
    pub(crate) fn section_missing_end_of_line(
        section_label_span: Span,
    ) -> impl Fn(TokenError) -> Self {
        move |error| {
            Self::new_from_token_error(
                error,
                ParserErrorReason::section_missing_end_of_line(section_label_span),
            )
        }
    }

    /// Creates a new `ParserError` for a missing identifier in a parameter
    pub(crate) fn parameter_missing_identifier(colon_token: Span) -> impl Fn(TokenError) -> Self {
        move |error| {
            Self::new_from_token_error(
                error,
                ParserErrorReason::parameter_missing_identifier(colon_token),
            )
        }
    }

    /// Creates a new `ParserError` for a missing equals sign in a parameter
    pub(crate) fn parameter_missing_equals_sign(ident_span: Span) -> impl Fn(TokenError) -> Self {
        move |error| {
            Self::new_from_token_error(
                error,
                ParserErrorReason::parameter_missing_equals_sign(ident_span),
            )
        }
    }

    /// Creates a new `ParserError` for a missing value in a parameter
    pub(crate) fn parameter_missing_value(equals_token: Span) -> impl Fn(Self) -> Self {
        move |error| error.convert_reason(ParserErrorReason::parameter_missing_value(equals_token))
    }

    /// Creates a new `ParserError` for a missing end of line in a parameter
    pub(crate) fn parameter_missing_end_of_line(value_span: Span) -> impl Fn(TokenError) -> Self {
        move |error| {
            Self::new_from_token_error(
                error,
                ParserErrorReason::parameter_missing_end_of_line(value_span),
            )
        }
    }

    /// Creates a new `ParserError` for a missing unit in a parameter
    pub(crate) fn parameter_missing_unit(colon_token: Span) -> impl Fn(Self) -> Self {
        move |error| error.convert_reason(ParserErrorReason::parameter_missing_unit(colon_token))
    }

    /// Creates a new `ParserError` for a missing minimum value in limits
    pub(crate) fn limit_missing_min(paren_left_token: Span) -> impl Fn(Self) -> Self {
        move |error| error.convert_reason(ParserErrorReason::limit_missing_min(paren_left_token))
    }

    /// Creates a new `ParserError` for a missing comma in limits
    pub(crate) fn limit_missing_comma(min_span: Span) -> impl Fn(TokenError) -> Self {
        move |error| {
            Self::new_from_token_error(error, ParserErrorReason::limit_missing_comma(min_span))
        }
    }

    /// Creates a new `ParserError` for a missing maximum value in limits
    pub(crate) fn limit_missing_max(comma_token: Span) -> impl Fn(Self) -> Self {
        move |error| error.convert_reason(ParserErrorReason::limit_missing_max(comma_token))
    }

    /// Creates a new `ParserError` for missing values in discrete limits
    pub(crate) fn limit_missing_values(bracket_left_token: Span) -> impl Fn(Self) -> Self {
        move |error| {
            error.convert_reason(ParserErrorReason::limit_missing_values(bracket_left_token))
        }
    }

    /// Creates a new `ParserError` for a missing expression in piecewise
    pub(crate) fn piecewise_missing_expr(brace_left_token: Span) -> impl Fn(Self) -> Self {
        move |error| {
            error.convert_reason(ParserErrorReason::piecewise_missing_expr(brace_left_token))
        }
    }

    /// Creates a new `ParserError` for a missing if keyword in piecewise
    pub(crate) fn piecewise_missing_if(expr_span: Span) -> impl Fn(TokenError) -> Self {
        move |error| {
            Self::new_from_token_error(error, ParserErrorReason::piecewise_missing_if(expr_span))
        }
    }

    /// Creates a new `ParserError` for a missing if expression in piecewise
    pub(crate) fn piecewise_missing_if_expr(if_token: Span) -> impl Fn(Self) -> Self {
        move |error| error.convert_reason(ParserErrorReason::piecewise_missing_if_expr(if_token))
    }

    /// Creates a new `ParserError` for a missing colon in a test declaration
    pub(crate) fn test_missing_colon(test_kw_or_inputs_span: Span) -> impl Fn(TokenError) -> Self {
        move |error| {
            Self::new_from_token_error(
                error,
                ParserErrorReason::test_missing_colon(test_kw_or_inputs_span),
            )
        }
    }

    /// Creates a new `ParserError` for a missing expression in a test declaration
    pub(crate) fn test_missing_expr(colon_token: Span) -> impl Fn(Self) -> Self {
        move |error| error.convert_reason(ParserErrorReason::test_missing_expr(colon_token))
    }

    /// Creates a new `ParserError` for a missing end of line in a test declaration
    pub(crate) fn test_missing_end_of_line(expr_span: Span) -> impl Fn(TokenError) -> Self {
        move |error| {
            Self::new_from_token_error(
                error,
                ParserErrorReason::test_missing_end_of_line(expr_span),
            )
        }
    }

    /// Creates a new `ParserError` for a missing second term in a unit expression
    pub(crate) fn unit_missing_second_term(operator_node: &UnitOpNode) -> impl Fn(Self) -> Self {
        move |error| {
            let operator_span = operator_node.span();
            let operator = **operator_node;
            error.convert_reason(ParserErrorReason::unit_missing_second_term(
                operator_span,
                operator,
            ))
        }
    }

    /// Creates a new `ParserError` for a missing exponent in a unit expression
    pub(crate) fn unit_missing_exponent(caret_token: Span) -> impl Fn(TokenError) -> Self {
        move |error| {
            Self::new_from_token_error(error, ParserErrorReason::unit_missing_exponent(caret_token))
        }
    }

    /// Creates a new `ParserError` for a missing expression in parenthesized unit
    pub(crate) fn unit_paren_missing_expr(paren_left_token: Span) -> impl Fn(Self) -> Self {
        move |error| {
            error.convert_reason(ParserErrorReason::unit_paren_missing_expr(paren_left_token))
        }
    }

    /// Creates a new `ParserError` for an unclosed bracket
    pub(crate) fn unclosed_bracket(bracket_left_token: Span) -> impl Fn(TokenError) -> Self {
        move |error| {
            Self::new_from_token_error(
                error,
                ParserErrorReason::unclosed_bracket(bracket_left_token),
            )
        }
    }

    /// Creates a new `ParserError` for an unclosed parenthesis
    pub(crate) fn unclosed_paren(paren_left_token: Span) -> impl Fn(TokenError) -> Self {
        move |error| {
            Self::new_from_token_error(error, ParserErrorReason::unclosed_paren(paren_left_token))
        }
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.reason)
    }
}

impl<'a> nom::error::ParseError<InputSpan<'a>> for ParserError {
    fn from_error_kind(input: InputSpan<'a>, kind: nom::error::ErrorKind) -> Self {
        #[expect(
            clippy::wildcard_enum_match_arm,
            reason = "this will only ever care about the EOF error kind"
        )]
        let reason = match kind {
            // If `all_consuming` is used, we expect the parser to consume the entire input
            nom::error::ErrorKind::Eof => ParserErrorReason::unexpected_token(),
            _ => ParserErrorReason::nom_error(kind),
        };

        Self {
            reason,
            error_offset: input.location_offset(),
        }
    }

    fn append(_input: InputSpan<'a>, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

/// Implements conversion from `TokenError` to `ParserError`.
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

impl AsOneilError for ParserError {
    fn message(&self) -> String {
        self.to_string()
    }

    fn error_location(&self, source: &str) -> Option<ErrorLocation> {
        let location = ErrorLocation::from_source_and_offset(source, self.error_offset);
        Some(location)
    }

    fn context_with_source(&self, source: &str) -> Vec<(Context, Option<ErrorLocation>)> {
        context::from_source(self.error_offset, &self.reason, source)
    }
}
