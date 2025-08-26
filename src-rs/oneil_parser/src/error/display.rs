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
            Self::ModelPathMissingSubcomponent => write!(f, "expected submodel name after `.`"),
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
            Self::MissingPath => write!(f, "expected model path after `use`"),
            Self::MissingAs => write!(f, "expected `as` after model path"),
            Self::MissingAlias => write!(f, "expected model alias after `as`"),
            Self::MissingEndOfLine => write!(f, "unexpected character"),
        }
    }
}

impl fmt::Display for ExprKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ComparisonOpMissingSecondOperand { operator } => {
                let operator_str = match operator {
                    oneil_ast::expression::ComparisonOp::LessThan => "<",
                    oneil_ast::expression::ComparisonOp::LessThanEq => "<=",
                    oneil_ast::expression::ComparisonOp::GreaterThan => ">",
                    oneil_ast::expression::ComparisonOp::GreaterThanEq => ">=",
                    oneil_ast::expression::ComparisonOp::Eq => "==",
                    oneil_ast::expression::ComparisonOp::NotEq => "!=",
                };
                write!(f, "expected operand after `{operator_str}`")
            }
            Self::BinaryOpMissingSecondOperand { operator } => {
                let operator_str = match operator {
                    oneil_ast::expression::BinaryOp::Add => "+",
                    oneil_ast::expression::BinaryOp::Sub => "-",
                    oneil_ast::expression::BinaryOp::TrueSub => "--",
                    oneil_ast::expression::BinaryOp::Mul => "*",
                    oneil_ast::expression::BinaryOp::Div => "/",
                    oneil_ast::expression::BinaryOp::TrueDiv => "//",
                    oneil_ast::expression::BinaryOp::Mod => "%",
                    oneil_ast::expression::BinaryOp::Pow => "^",
                    oneil_ast::expression::BinaryOp::And => "&&",
                    oneil_ast::expression::BinaryOp::Or => "||",
                    oneil_ast::expression::BinaryOp::MinMax => "|",
                };
                write!(f, "expected operand after `{operator_str}`")
            }
            Self::ParenMissingExpr => write!(f, "expected expression inside parentheses"),
            Self::UnaryOpMissingOperand { operator } => {
                let operator_str = match operator {
                    oneil_ast::expression::UnaryOp::Neg => "-",
                    oneil_ast::expression::UnaryOp::Not => "!",
                };
                write!(f, "expected operand after `{operator_str}`")
            }
            Self::VariableMissingParentModel => write!(f, "expected parent model name after `.`"),
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
                    oneil_ast::unit::UnitOp::Multiply => "*",
                    oneil_ast::unit::UnitOp::Divide => "/",
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
            _ => write!(
                f,
                "unexpected token error `{self:?}`. please submit an issue at <https://github.com/oneil-lang/oneil/issues>"
            ),
        }
    }
}
