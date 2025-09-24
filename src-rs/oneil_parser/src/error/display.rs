use oneil_ast::{BinaryOp, ComparisonOp, UnaryOp, UnitOp};

use crate::{
    error::reason::{
        DeclKind, ExpectKind, ExprKind, ImportKind, IncompleteKind, ParameterKind,
        ParserErrorReason, SectionKind, TestKind, UnitKind, UseKind,
    },
    token::error::{IncompleteKind as TokenIncompleteKind, TokenErrorKind},
};
use std::fmt;

impl fmt::Display for ParserErrorReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Expect(expect_kind) => expect_kind.fmt(f),
            Self::Incomplete { cause: _, kind } => kind.fmt(f),
            Self::UnexpectedToken => write!(f, "unexpected token"),
            Self::TokenError(token_error_kind) => token_error_kind.fmt(f),

            #[expect(
                clippy::use_debug,
                reason = "a debug output the best output we can give here"
            )]
            Self::NomError(error_kind) => {
                write!(
                    f,
                    "unexpected nom parser error `{error_kind:?}`. please submit an issue at <https://github.com/oneil-lang/oneil/issues>"
                )
            }
        }
    }
}

impl fmt::Display for ExpectKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // simplified from "expected declaration" in order to make the error
            // message more understandable for the average user
            Self::Decl => write!(f, "expected parameter or test"),
            Self::Expr => write!(f, "expected expression"),
            Self::Note => write!(f, "expected note"),
            Self::Parameter => write!(f, "expected parameter"),
            Self::Test => write!(f, "expected test"),
            Self::Unit => write!(f, "expected unit"),
        }
    }
}

impl fmt::Display for DeclKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Import(import_kind) => import_kind.fmt(f),
            Self::Use(use_kind) => use_kind.fmt(f),
            Self::ModelMissingSubcomponent => write!(f, "expected submodel name after `.`"),
            Self::AsMissingAlias => write!(f, "expected model alias after `as`"),
        }
    }
}

impl fmt::Display for ImportKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingPath => write!(f, "expected path after `import`"),
            Self::MissingEndOfLine => write!(f, "unexpected character"),
        }
    }
}

impl fmt::Display for UseKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingModelInfo => write!(f, "expected model after `use`"),
            Self::MissingEndOfLine => write!(f, "unexpected character"),
        }
    }
}

impl fmt::Display for ExprKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ComparisonOpMissingSecondOperand { operator } => {
                let operator_str = match operator {
                    ComparisonOp::LessThan => "<",
                    ComparisonOp::LessThanEq => "<=",
                    ComparisonOp::GreaterThan => ">",
                    ComparisonOp::GreaterThanEq => ">=",
                    ComparisonOp::Eq => "==",
                    ComparisonOp::NotEq => "!=",
                };
                write!(f, "expected operand after `{operator_str}`")
            }
            Self::BinaryOpMissingSecondOperand { operator } => {
                let operator_str = match operator {
                    BinaryOp::Add => "+",
                    BinaryOp::Sub => "-",
                    BinaryOp::TrueSub => "--",
                    BinaryOp::Mul => "*",
                    BinaryOp::Div => "/",
                    BinaryOp::TrueDiv => "//",
                    BinaryOp::Mod => "%",
                    BinaryOp::Pow => "^",
                    BinaryOp::And => "&&",
                    BinaryOp::Or => "||",
                    BinaryOp::MinMax => "|",
                };
                write!(f, "expected operand after `{operator_str}`")
            }
            Self::ParenMissingExpr => write!(f, "expected expression inside parentheses"),
            Self::UnaryOpMissingOperand { operator } => {
                let operator_str = match operator {
                    UnaryOp::Neg => "-",
                    UnaryOp::Not => "!",
                };
                write!(f, "expected operand after `{operator_str}`")
            }
            Self::VariableMissingReferenceModel => {
                write!(f, "expected parent model name after `.`")
            }
        }
    }
}

impl fmt::Display for SectionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingLabel => write!(f, "expected label after `section`"),
            Self::MissingEndOfLine => write!(f, "unexpected character"),
        }
    }
}

impl fmt::Display for ParameterKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingIdentifier => write!(f, "expected parameter identifier"),
            Self::MissingEqualsSign => write!(f, "expected `=`"),
            Self::MissingValue => write!(f, "expected parameter value after `=`"),
            Self::MissingEndOfLine => write!(f, "unexpected character"),
            Self::MissingUnit => write!(f, "expected unit after `:`"),
            Self::LimitMissingMin => write!(f, "expected limit minimum value"),
            Self::LimitMissingComma => write!(f, "expected `,`"),
            Self::LimitMissingMax => write!(f, "expected limit maximum value"),
            Self::LimitMissingValues => write!(f, "expected limit values"),
            Self::PiecewiseMissingExpr => write!(f, "expected piecewise expression"),
            Self::PiecewiseMissingIf => write!(f, "expected `if`"),
            Self::PiecewiseMissingIfExpr => {
                write!(f, "expected piecewise conditional expression after `if`")
            }
        }
    }
}

impl fmt::Display for TestKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingColon => write!(f, "expected `:`"),
            Self::MissingExpr => write!(f, "expected test expression"),
            Self::MissingEndOfLine => write!(f, "unexpected character"),
        }
    }
}

impl fmt::Display for UnitKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSecondTerm { operator } => {
                let operator_str = match operator {
                    UnitOp::Multiply => "*",
                    UnitOp::Divide => "/",
                };
                write!(f, "expected second operand after `{operator_str}`")
            }
            Self::MissingExponent => write!(f, "expected exponent"),
            Self::ParenMissingExpr => write!(f, "expected expression inside parentheses"),
        }
    }
}

impl fmt::Display for IncompleteKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Decl(decl_kind) => decl_kind.fmt(f),
            Self::Expr(expr_kind) => expr_kind.fmt(f),
            Self::Section(section_kind) => section_kind.fmt(f),
            Self::Parameter(parameter_kind) => parameter_kind.fmt(f),
            Self::Test(test_kind) => test_kind.fmt(f),
            Self::Unit(unit_kind) => unit_kind.fmt(f),
            Self::UnclosedBracket => write!(f, "unclosed `[`"),
            Self::UnclosedParen => write!(f, "unclosed `(`"),
        }
    }
}

impl fmt::Display for TokenErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Incomplete(incomplete_kind) => match incomplete_kind {
                TokenIncompleteKind::UnclosedNote {
                    delimiter_start_offset: _,
                    delimiter_length: _,
                } => write!(f, "unclosed note"),
                TokenIncompleteKind::UnclosedString {
                    open_quote_offset: _,
                } => write!(f, "unclosed string"),
                TokenIncompleteKind::InvalidDecimalPart {
                    decimal_point_offset: _,
                } => write!(f, "invalid decimal part"),
                TokenIncompleteKind::InvalidExponentPart { e_offset: _ } => {
                    write!(f, "invalid exponent part")
                }
            },

            #[expect(
                clippy::use_debug,
                reason = "a debug output the best output we can give here"
            )]
            error @ (Self::Expect(_) | Self::NomError(_)) => write!(
                f,
                "unexpected token error `{error:?}`. please submit an issue at <https://github.com/oneil-lang/oneil/issues>"
            ),
        }
    }
}
