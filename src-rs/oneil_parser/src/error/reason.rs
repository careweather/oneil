//! Detailed error reasons and categories for parser errors.
//!
//! This module contains the `ParserErrorReason` enum and related types that
//! provide detailed categorization of parsing errors. It includes specific
//! error types for different language constructs like declarations, expressions,
//! parameters, and units.
//!
//! # Error Categories
//!
//! - **Expect**: Expected a specific language construct but found something else
//! - **Incomplete**: Found an incomplete input with specific details about what's missing
//! - **UnexpectedToken**: Found a token that wasn't expected in the current context
//! - **TokenError**: Low-level tokenization errors
//! - **NomError**: Internal nom parsing library errors
use oneil_ast::{
    Span as AstSpan,
    expression::{BinaryOp, UnaryOp},
    unit::UnitOp,
};

use crate::token::error::TokenErrorKind;

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
        /// The span in the source code that indicated that the input was incomplete
        cause: AstSpan,
        /// The specific type of incomplete input that was found
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
    pub(crate) fn expect_decl() -> Self {
        Self::Expect(ExpectKind::Decl)
    }

    pub(crate) fn expect_expr() -> Self {
        Self::Expect(ExpectKind::Expr)
    }

    pub(crate) fn expect_note() -> Self {
        Self::Expect(ExpectKind::Note)
    }

    pub(crate) fn expect_parameter() -> Self {
        Self::Expect(ExpectKind::Parameter)
    }

    pub(crate) fn expect_test() -> Self {
        Self::Expect(ExpectKind::Test)
    }

    pub(crate) fn expect_unit() -> Self {
        Self::Expect(ExpectKind::Unit)
    }

    pub(crate) fn import_missing_path(import_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: import_span,
            kind: IncompleteKind::Decl(DeclKind::Import(ImportKind::MissingPath)),
        }
    }

    pub(crate) fn import_missing_end_of_line(import_path_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: import_path_span,
            kind: IncompleteKind::Decl(DeclKind::Import(ImportKind::MissingEndOfLine)),
        }
    }

    pub(crate) fn from_missing_path(from_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: from_span,
            kind: IncompleteKind::Decl(DeclKind::From(FromKind::MissingPath)),
        }
    }

    pub(crate) fn from_missing_use(from_path_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: from_path_span,
            kind: IncompleteKind::Decl(DeclKind::From(FromKind::MissingUse)),
        }
    }

    pub(crate) fn from_missing_use_model(use_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: use_span,
            kind: IncompleteKind::Decl(DeclKind::From(FromKind::MissingUseModel)),
        }
    }

    pub(crate) fn from_missing_as(use_model_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: use_model_span,
            kind: IncompleteKind::Decl(DeclKind::From(FromKind::MissingAs)),
        }
    }

    pub(crate) fn from_missing_alias(as_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: as_span,
            kind: IncompleteKind::Decl(DeclKind::From(FromKind::MissingAlias)),
        }
    }

    pub(crate) fn from_missing_end_of_line(alias_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: alias_span,
            kind: IncompleteKind::Decl(DeclKind::From(FromKind::MissingEndOfLine)),
        }
    }

    pub(crate) fn use_missing_path(use_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: use_span,
            kind: IncompleteKind::Decl(DeclKind::Use(UseKind::MissingPath)),
        }
    }

    pub(crate) fn use_missing_as(use_path_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: use_path_span,
            kind: IncompleteKind::Decl(DeclKind::Use(UseKind::MissingAs)),
        }
    }

    pub(crate) fn use_missing_alias(as_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: as_span,
            kind: IncompleteKind::Decl(DeclKind::Use(UseKind::MissingAlias)),
        }
    }

    pub(crate) fn use_missing_end_of_line(alias_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: alias_span,
            kind: IncompleteKind::Decl(DeclKind::Use(UseKind::MissingEndOfLine)),
        }
    }

    pub(crate) fn model_path_missing_subcomponent(dot_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: dot_span,
            kind: IncompleteKind::Decl(DeclKind::ModelPathMissingSubcomponent),
        }
    }

    pub(crate) fn expr_binary_op_missing_second_operand(
        operator_span: AstSpan,
        operator: BinaryOp,
    ) -> Self {
        Self::Incomplete {
            cause: operator_span,
            kind: IncompleteKind::Expr(ExprKind::BinaryOpMissingSecondOperand { operator }),
        }
    }

    pub(crate) fn expr_paren_missing_expr(paren_left_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: paren_left_span,
            kind: IncompleteKind::Expr(ExprKind::ParenMissingExpr),
        }
    }

    pub(crate) fn expr_unary_op_missing_operand(operator_span: AstSpan, operator: UnaryOp) -> Self {
        Self::Incomplete {
            cause: operator_span,
            kind: IncompleteKind::Expr(ExprKind::UnaryOpMissingOperand { operator }),
        }
    }

    pub(crate) fn expr_variable_missing_parent_model(dot_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: dot_span,
            kind: IncompleteKind::Expr(ExprKind::VariableMissingParentModel),
        }
    }

    pub(crate) fn section_missing_label(section_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: section_span,
            kind: IncompleteKind::Section(SectionKind::MissingLabel),
        }
    }

    pub(crate) fn section_missing_end_of_line(section_label_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: section_label_span,
            kind: IncompleteKind::Section(SectionKind::MissingEndOfLine),
        }
    }

    pub(crate) fn parameter_missing_identifier(colon_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: colon_span,
            kind: IncompleteKind::Parameter(ParameterKind::MissingIdentifier),
        }
    }

    pub(crate) fn parameter_missing_equals_sign(ident_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: ident_span,
            kind: IncompleteKind::Parameter(ParameterKind::MissingEqualsSign),
        }
    }

    pub(crate) fn parameter_missing_value(equals_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: equals_span,
            kind: IncompleteKind::Parameter(ParameterKind::MissingValue),
        }
    }

    pub(crate) fn parameter_missing_end_of_line(value_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: value_span,
            kind: IncompleteKind::Parameter(ParameterKind::MissingEndOfLine),
        }
    }

    pub(crate) fn parameter_missing_unit(colon_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: colon_span,
            kind: IncompleteKind::Parameter(ParameterKind::MissingUnit),
        }
    }

    pub(crate) fn limit_missing_min(left_paren_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: left_paren_span,
            kind: IncompleteKind::Parameter(ParameterKind::LimitMissingMin),
        }
    }

    pub(crate) fn limit_missing_comma(min_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: min_span,
            kind: IncompleteKind::Parameter(ParameterKind::LimitMissingComma),
        }
    }

    pub(crate) fn limit_missing_max(comma_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: comma_span,
            kind: IncompleteKind::Parameter(ParameterKind::LimitMissingMax),
        }
    }

    pub(crate) fn limit_missing_values(left_bracket_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: left_bracket_span,
            kind: IncompleteKind::Parameter(ParameterKind::LimitMissingValues),
        }
    }

    pub(crate) fn piecewise_missing_expr(brace_left_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: brace_left_span,
            kind: IncompleteKind::Parameter(ParameterKind::PiecewiseMissingExpr),
        }
    }

    pub(crate) fn piecewise_missing_if(expr_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: expr_span,
            kind: IncompleteKind::Parameter(ParameterKind::PiecewiseMissingIf),
        }
    }

    pub(crate) fn piecewise_missing_if_expr(if_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: if_span,
            kind: IncompleteKind::Parameter(ParameterKind::PiecewiseMissingIfExpr),
        }
    }

    pub(crate) fn test_missing_colon(test_kw_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: test_kw_span,
            kind: IncompleteKind::Test(TestKind::MissingColon),
        }
    }

    pub(crate) fn test_missing_expr(colon_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: colon_span,
            kind: IncompleteKind::Test(TestKind::MissingExpr),
        }
    }

    pub(crate) fn test_missing_end_of_line(expr_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: expr_span,
            kind: IncompleteKind::Test(TestKind::MissingEndOfLine),
        }
    }

    pub(crate) fn unit_missing_second_term(operator_span: AstSpan, operator: UnitOp) -> Self {
        Self::Incomplete {
            cause: operator_span,
            kind: IncompleteKind::Unit(UnitKind::MissingSecondTerm { operator }),
        }
    }

    pub(crate) fn unit_missing_exponent(caret_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: caret_span,
            kind: IncompleteKind::Unit(UnitKind::MissingExponent),
        }
    }

    pub(crate) fn unit_paren_missing_expr(paren_left_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: paren_left_span,
            kind: IncompleteKind::Unit(UnitKind::ParenMissingExpr),
        }
    }

    pub(crate) fn unclosed_bracket(bracket_left_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: bracket_left_span,
            kind: IncompleteKind::UnclosedBracket,
        }
    }

    pub(crate) fn unclosed_paren(paren_left_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: paren_left_span,
            kind: IncompleteKind::UnclosedParen,
        }
    }

    pub(crate) fn unexpected_token() -> Self {
        Self::UnexpectedToken
    }

    pub(crate) fn token_error(kind: TokenErrorKind) -> Self {
        Self::TokenError(kind)
    }

    pub(crate) fn nom_error(kind: nom::error::ErrorKind) -> Self {
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
