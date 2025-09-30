//! Detailed error reasons and categories for parser errors.
use oneil_ast::{AstSpan, BinaryOp, ComparisonOp, UnaryOp, UnitOp};

use crate::token::error::TokenErrorKind;

/// The different kinds of errors that can occur during parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    #[must_use]
    pub(crate) const fn expect_decl() -> Self {
        Self::Expect(ExpectKind::Decl)
    }

    #[must_use]
    pub(crate) const fn expect_expr() -> Self {
        Self::Expect(ExpectKind::Expr)
    }

    #[must_use]
    pub(crate) const fn expect_note() -> Self {
        Self::Expect(ExpectKind::Note)
    }

    #[must_use]
    pub(crate) const fn expect_parameter() -> Self {
        Self::Expect(ExpectKind::Parameter)
    }

    #[must_use]
    pub(crate) const fn expect_test() -> Self {
        Self::Expect(ExpectKind::Test)
    }

    #[must_use]
    pub(crate) const fn expect_unit() -> Self {
        Self::Expect(ExpectKind::Unit)
    }

    #[must_use]
    pub(crate) const fn import_missing_path(import_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: import_span,
            kind: IncompleteKind::Decl(DeclKind::Import(ImportKind::MissingPath)),
        }
    }

    #[must_use]
    pub(crate) const fn import_missing_end_of_line(import_path_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: import_path_span,
            kind: IncompleteKind::Decl(DeclKind::Import(ImportKind::MissingEndOfLine)),
        }
    }

    #[must_use]
    pub(crate) const fn use_missing_model_info(use_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: use_span,
            kind: IncompleteKind::Decl(DeclKind::Use(UseKind::MissingModelInfo)),
        }
    }

    #[must_use]
    pub(crate) const fn as_missing_alias(as_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: as_span,
            kind: IncompleteKind::Decl(DeclKind::AsMissingAlias),
        }
    }

    #[must_use]
    pub(crate) const fn use_missing_end_of_line(alias_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: alias_span,
            kind: IncompleteKind::Decl(DeclKind::Use(UseKind::MissingEndOfLine)),
        }
    }

    #[must_use]
    pub(crate) const fn model_path_missing_subcomponent(dot_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: dot_span,
            kind: IncompleteKind::Decl(DeclKind::ModelMissingSubcomponent),
        }
    }

    #[must_use]
    pub(crate) const fn expr_comparison_op_missing_second_operand(
        operator_span: AstSpan,
        operator: ComparisonOp,
    ) -> Self {
        Self::Incomplete {
            cause: operator_span,
            kind: IncompleteKind::Expr(ExprKind::ComparisonOpMissingSecondOperand { operator }),
        }
    }

    #[must_use]
    pub(crate) const fn expr_binary_op_missing_second_operand(
        operator_span: AstSpan,
        operator: BinaryOp,
    ) -> Self {
        Self::Incomplete {
            cause: operator_span,
            kind: IncompleteKind::Expr(ExprKind::BinaryOpMissingSecondOperand { operator }),
        }
    }

    #[must_use]
    pub(crate) const fn expr_paren_missing_expr(paren_left_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: paren_left_span,
            kind: IncompleteKind::Expr(ExprKind::ParenMissingExpr),
        }
    }

    #[must_use]
    pub(crate) const fn expr_unary_op_missing_operand(
        operator_span: AstSpan,
        operator: UnaryOp,
    ) -> Self {
        Self::Incomplete {
            cause: operator_span,
            kind: IncompleteKind::Expr(ExprKind::UnaryOpMissingOperand { operator }),
        }
    }

    #[must_use]
    pub(crate) const fn expr_variable_missing_reference_model(dot_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: dot_span,
            kind: IncompleteKind::Expr(ExprKind::VariableMissingReferenceModel),
        }
    }

    #[must_use]
    pub(crate) const fn section_missing_label(section_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: section_span,
            kind: IncompleteKind::Section(SectionKind::MissingLabel),
        }
    }

    #[must_use]
    pub(crate) const fn section_missing_end_of_line(section_label_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: section_label_span,
            kind: IncompleteKind::Section(SectionKind::MissingEndOfLine),
        }
    }

    #[must_use]
    pub(crate) const fn parameter_missing_identifier(colon_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: colon_span,
            kind: IncompleteKind::Parameter(ParameterKind::MissingIdentifier),
        }
    }

    #[must_use]
    pub(crate) const fn parameter_missing_equals_sign(ident_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: ident_span,
            kind: IncompleteKind::Parameter(ParameterKind::MissingEqualsSign),
        }
    }

    #[must_use]
    pub(crate) const fn parameter_missing_value(equals_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: equals_span,
            kind: IncompleteKind::Parameter(ParameterKind::MissingValue),
        }
    }

    #[must_use]
    pub(crate) const fn parameter_missing_end_of_line(value_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: value_span,
            kind: IncompleteKind::Parameter(ParameterKind::MissingEndOfLine),
        }
    }

    #[must_use]
    pub(crate) const fn parameter_missing_unit(colon_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: colon_span,
            kind: IncompleteKind::Parameter(ParameterKind::MissingUnit),
        }
    }

    #[must_use]
    pub(crate) const fn limit_missing_min(left_paren_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: left_paren_span,
            kind: IncompleteKind::Parameter(ParameterKind::LimitMissingMin),
        }
    }

    #[must_use]
    pub(crate) const fn limit_missing_comma(min_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: min_span,
            kind: IncompleteKind::Parameter(ParameterKind::LimitMissingComma),
        }
    }

    #[must_use]
    pub(crate) const fn limit_missing_max(comma_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: comma_span,
            kind: IncompleteKind::Parameter(ParameterKind::LimitMissingMax),
        }
    }

    #[must_use]
    pub(crate) const fn limit_missing_values(left_bracket_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: left_bracket_span,
            kind: IncompleteKind::Parameter(ParameterKind::LimitMissingValues),
        }
    }

    #[must_use]
    pub(crate) const fn piecewise_missing_expr(brace_left_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: brace_left_span,
            kind: IncompleteKind::Parameter(ParameterKind::PiecewiseMissingExpr),
        }
    }

    #[must_use]
    pub(crate) const fn piecewise_missing_if(expr_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: expr_span,
            kind: IncompleteKind::Parameter(ParameterKind::PiecewiseMissingIf),
        }
    }

    #[must_use]
    pub(crate) const fn piecewise_missing_if_expr(if_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: if_span,
            kind: IncompleteKind::Parameter(ParameterKind::PiecewiseMissingIfExpr),
        }
    }

    #[must_use]
    pub(crate) const fn test_missing_colon(test_kw_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: test_kw_span,
            kind: IncompleteKind::Test(TestKind::MissingColon),
        }
    }

    #[must_use]
    pub(crate) const fn test_missing_expr(colon_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: colon_span,
            kind: IncompleteKind::Test(TestKind::MissingExpr),
        }
    }

    #[must_use]
    pub(crate) const fn test_missing_end_of_line(expr_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: expr_span,
            kind: IncompleteKind::Test(TestKind::MissingEndOfLine),
        }
    }

    #[must_use]
    pub(crate) const fn unit_missing_second_term(operator_span: AstSpan, operator: UnitOp) -> Self {
        Self::Incomplete {
            cause: operator_span,
            kind: IncompleteKind::Unit(UnitKind::MissingSecondTerm { operator }),
        }
    }

    #[must_use]
    pub(crate) const fn unit_missing_exponent(caret_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: caret_span,
            kind: IncompleteKind::Unit(UnitKind::MissingExponent),
        }
    }

    #[must_use]
    pub(crate) const fn unit_paren_missing_expr(paren_left_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: paren_left_span,
            kind: IncompleteKind::Unit(UnitKind::ParenMissingExpr),
        }
    }

    #[must_use]
    pub(crate) const fn unclosed_bracket(bracket_left_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: bracket_left_span,
            kind: IncompleteKind::UnclosedBracket,
        }
    }

    #[must_use]
    pub(crate) const fn unclosed_paren(paren_left_span: AstSpan) -> Self {
        Self::Incomplete {
            cause: paren_left_span,
            kind: IncompleteKind::UnclosedParen,
        }
    }

    #[must_use]
    pub(crate) const fn unexpected_token() -> Self {
        Self::UnexpectedToken
    }

    #[must_use]
    pub(crate) const fn token_error(kind: TokenErrorKind) -> Self {
        Self::TokenError(kind)
    }

    #[must_use]
    pub(crate) const fn nom_error(kind: nom::error::ErrorKind) -> Self {
        Self::NomError(kind)
    }
}

/// The different kinds of AST nodes that can be expected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclKind {
    /// Found an incomplete `import` declaration
    Import(
        /// The kind of import error
        ImportKind,
    ),
    /// Found an incomplete `use` declaration
    Use(
        /// The kind of use error
        UseKind,
    ),
    /// Found an incomplete model path
    ModelMissingSubcomponent,
    /// Found an incomplete alias after `as`
    AsMissingAlias,
}

/// The different kind of `import` errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportKind {
    /// Found an incomplete path
    MissingPath,
    /// Missing end of line
    MissingEndOfLine,
}

/// The different kind of `use` errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UseKind {
    /// Missing the model info
    MissingModelInfo,
    /// Missing end of line
    MissingEndOfLine,
}

/// The different kind of incomplete expression errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExprKind {
    /// Found a comparison operation missing a second operand
    ComparisonOpMissingSecondOperand {
        /// The operator value
        operator: ComparisonOp,
    },
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
    /// Found a missing reference model in a variable accessor
    VariableMissingReferenceModel,
}

/// The different kind of incomplete section errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionKind {
    /// Found an incomplete section label
    MissingLabel,
    /// Missing end of line
    MissingEndOfLine,
}

/// The different kind of incomplete parameter errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestKind {
    /// Found a missing colon in a test declaration
    MissingColon,
    /// Found a missing expression in a test declaration
    MissingExpr,
    /// Found a missing end of line in a test declaration
    MissingEndOfLine,
}

/// The different kind of incomplete unit errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
