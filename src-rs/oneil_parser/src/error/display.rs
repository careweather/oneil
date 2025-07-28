use crate::error::reason::{
    DeclKind, ExpectKind, ExprKind, FromKind, ImportKind, IncompleteKind, ParameterKind,
    ParserErrorReason, SectionKind, TestKind, UnitKind, UseKind,
};

pub fn reason_to_string(reason: &ParserErrorReason) -> String {
    match &reason {
        ParserErrorReason::Expect(expect_kind) => match expect_kind {
            // simplified from "expected declaration" in order to make the error
            // message more understandable for the average user
            ExpectKind::Decl => "expected parameter or test".to_string(),
            ExpectKind::Expr => "expected expression".to_string(),
            ExpectKind::Note => "expected note".to_string(),
            ExpectKind::Parameter => "expected parameter".to_string(),
            ExpectKind::Test => "expected test".to_string(),
            ExpectKind::Unit => "expected unit".to_string(),
        },
        ParserErrorReason::Incomplete { cause: _, kind } => match kind {
            IncompleteKind::Decl(decl_kind) => match decl_kind {
                DeclKind::Import(import_kind) => match import_kind {
                    ImportKind::MissingPath => "expected path after `import`".to_string(),
                    ImportKind::MissingEndOfLine => {
                        "import declaration must be followed by a new line".to_string()
                    }
                },
                DeclKind::From(from_kind) => match from_kind {
                    FromKind::MissingPath => "expected path after `from`".to_string(),
                    FromKind::MissingUse => "expected `use` after path".to_string(),
                    FromKind::MissingUseModel => "expected model after `use`".to_string(),
                    FromKind::MissingAs => "expected `as` after model".to_string(),
                    FromKind::MissingAlias => "expected model alias after `as`".to_string(),
                    FromKind::MissingEndOfLine => {
                        "from declaration must be followed by a new line".to_string()
                    }
                },
                DeclKind::Use(use_kind) => match use_kind {
                    UseKind::MissingPath => "expected path after `use`".to_string(),
                    UseKind::MissingAs => "expected `as` after path".to_string(),
                    UseKind::MissingAlias => "expected model alias after `as`".to_string(),
                    UseKind::MissingEndOfLine => {
                        "use declaration must be followed by a new line".to_string()
                    }
                },
                DeclKind::ModelInputMissingEquals => "expected `=`".to_string(),
                DeclKind::ModelInputMissingValue => "expected value after `=`".to_string(),
                DeclKind::ModelPathMissingSubcomponent => "expected model after `.`".to_string(),
            },
            IncompleteKind::Expr(expr_kind) => match expr_kind {
                ExprKind::BinaryOpMissingSecondOperand { operator } => {
                    let operator_str = match operator {
                        oneil_ast::expression::BinaryOp::Add => "+".to_string(),
                        oneil_ast::expression::BinaryOp::Sub => "-".to_string(),
                        oneil_ast::expression::BinaryOp::TrueSub => "--".to_string(),
                        oneil_ast::expression::BinaryOp::Mul => "*".to_string(),
                        oneil_ast::expression::BinaryOp::Div => "/".to_string(),
                        oneil_ast::expression::BinaryOp::TrueDiv => "//".to_string(),
                        oneil_ast::expression::BinaryOp::Mod => "%".to_string(),
                        oneil_ast::expression::BinaryOp::Pow => "^".to_string(),
                        oneil_ast::expression::BinaryOp::LessThan => "<".to_string(),
                        oneil_ast::expression::BinaryOp::LessThanEq => "<=".to_string(),
                        oneil_ast::expression::BinaryOp::GreaterThan => ">".to_string(),
                        oneil_ast::expression::BinaryOp::GreaterThanEq => ">=".to_string(),
                        oneil_ast::expression::BinaryOp::Eq => "==".to_string(),
                        oneil_ast::expression::BinaryOp::NotEq => "!=".to_string(),
                        oneil_ast::expression::BinaryOp::And => "&&".to_string(),
                        oneil_ast::expression::BinaryOp::Or => "||".to_string(),
                        oneil_ast::expression::BinaryOp::MinMax => "|".to_string(),
                    };
                    format!("expected second operand after `{}`", operator_str)
                }
                ExprKind::ParenMissingExpr => "expected expression inside parentheses".to_string(),
                ExprKind::UnaryOpMissingOperand { operator } => {
                    let operator_str = match operator {
                        oneil_ast::expression::UnaryOp::Neg => "-".to_string(),
                        oneil_ast::expression::UnaryOp::Not => "!".to_string(),
                    };
                    format!("expected operand after `{}`", operator_str)
                }
                ExprKind::VariableMissingParentModel => {
                    "expected parent model after `.`".to_string()
                }
            },
            IncompleteKind::Section(section_kind) => match section_kind {
                SectionKind::MissingLabel => "expected label after `section`".to_string(),
                SectionKind::MissingEndOfLine => {
                    "section must be followed by a new line".to_string()
                }
            },
            IncompleteKind::Parameter(parameter_kind) => match parameter_kind {
                ParameterKind::MissingIdentifier => "expected identifier".to_string(),
                ParameterKind::MissingEqualsSign => "expected `=`".to_string(),
                ParameterKind::MissingValue => "expected value after `=`".to_string(),
                ParameterKind::MissingEndOfLine => {
                    "parameter must be followed by a new line".to_string()
                }
                ParameterKind::MissingUnit => "expected unit after `:`".to_string(),
                ParameterKind::LimitMissingMin => "expected minimum value".to_string(),
                ParameterKind::LimitMissingComma => "expected `,`".to_string(),
                ParameterKind::LimitMissingMax => "expected maximum value".to_string(),
                ParameterKind::LimitMissingValues => "expected values".to_string(),
                ParameterKind::PiecewiseMissingExpr => "expected expression".to_string(),
                ParameterKind::PiecewiseMissingIf => "expected `if`".to_string(),
                ParameterKind::PiecewiseMissingIfExpr => {
                    "piecewise missing conditional expression after `if`".to_string()
                }
            },
            IncompleteKind::Test(test_kind) => match test_kind {
                TestKind::MissingColon => "expected `:`".to_string(),
                TestKind::MissingExpr => "expected test expression".to_string(),
                TestKind::MissingEndOfLine => "test must be followed by a new line".to_string(),
                TestKind::MissingInputs => "expected inputs in `{}`".to_string(),
            },
            IncompleteKind::Unit(unit_kind) => match unit_kind {
                UnitKind::MissingSecondTerm { operator } => {
                    let operator_str = match operator {
                        oneil_ast::unit::UnitOp::Multiply => "*".to_string(),
                        oneil_ast::unit::UnitOp::Divide => "/".to_string(),
                    };
                    format!("expected second operand after `{}`", operator_str)
                }
                UnitKind::MissingExponent => "expected exponent".to_string(),
                UnitKind::ParenMissingExpr => "expected expression inside parentheses".to_string(),
            },
            IncompleteKind::UnclosedBrace => "unclosed `{`".to_string(),
            IncompleteKind::UnclosedBracket => "unclosed `[`".to_string(),
            IncompleteKind::UnclosedParen => "unclosed `(`".to_string(),
        },
        ParserErrorReason::UnexpectedToken => "unexpected token".to_string(),
        ParserErrorReason::TokenError(token_error_kind) => {
            format!(
                "unexpected token error `{:?}`. please submit an issue at https://github.com/oneil-lang/oneil/issues",
                token_error_kind
            )
        }
        ParserErrorReason::NomError(error_kind) => {
            format!(
                "unexpected nom parser error `{:?}`. please submit an issue at https://github.com/oneil-lang/oneil/issues",
                error_kind
            )
        }
    }
}
