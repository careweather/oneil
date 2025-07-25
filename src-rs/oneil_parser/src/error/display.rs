use crate::error::reason::{
    DeclKind, ExpectKind, ExprKind, FromKind, ImportKind, IncompleteKind, ParameterKind,
    ParserErrorReason, SectionKind, TestKind, UnitKind, UseKind,
};

pub fn reason_to_string(reason: &ParserErrorReason) -> String {
    match &reason {
        ParserErrorReason::Expect(expect_kind) => match expect_kind {
            ExpectKind::Decl => "expected declaration".to_string(),
            ExpectKind::Expr => "expected expression".to_string(),
            ExpectKind::Note => "expected note".to_string(),
            ExpectKind::Parameter => "expected parameter".to_string(),
            ExpectKind::Test => "expected test".to_string(),
            ExpectKind::Unit => "expected unit".to_string(),
        },
        ParserErrorReason::Incomplete { cause: _, kind } => match kind {
            IncompleteKind::Decl(decl_kind) => match decl_kind {
                DeclKind::Import(import_kind) => match import_kind {
                    ImportKind::MissingPath => "import declaration missing path".to_string(),
                    ImportKind::MissingEndOfLine => {
                        "import declaration must be followed by a new line".to_string()
                    }
                },
                DeclKind::From(from_kind) => match from_kind {
                    FromKind::MissingPath => "from declaration missing path".to_string(),
                    FromKind::MissingUse => "from declaration missing `use`".to_string(),
                    FromKind::MissingUseModel => {
                        "from declaration missing model after `use`".to_string()
                    }
                    FromKind::MissingAs => "from declaration missing `as`".to_string(),
                    FromKind::MissingAlias => {
                        "from declaration missing model alias after `as`".to_string()
                    }
                    FromKind::MissingEndOfLine => {
                        "from declaration must be followed by a new line".to_string()
                    }
                },
                DeclKind::Use(use_kind) => match use_kind {
                    UseKind::MissingPath => "use declaration missing path".to_string(),
                    UseKind::MissingAs => "use declaration missing `as`".to_string(),
                    UseKind::MissingAlias => {
                        "use declaration missing model alias after `as`".to_string()
                    }
                    UseKind::MissingEndOfLine => {
                        "use declaration must be followed by a new line".to_string()
                    }
                },
                DeclKind::ModelInputMissingEquals => "model input missing `=`".to_string(),
                DeclKind::ModelInputMissingValue => "model input missing value".to_string(),
                DeclKind::ModelPathMissingSubcomponent => {
                    "model path missing model after `.`".to_string()
                }
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
                    format!("operator `{}` missing second operand", operator_str)
                }
                ExprKind::ParenMissingExpr => {
                    "parenthesized expression missing expression".to_string()
                }
                ExprKind::UnaryOpMissingOperand { operator } => {
                    let operator_str = match operator {
                        oneil_ast::expression::UnaryOp::Neg => "-".to_string(),
                        oneil_ast::expression::UnaryOp::Not => "!".to_string(),
                    };
                    format!("operator `{}` missing operand", operator_str)
                }
                ExprKind::VariableMissingParentModel => {
                    "variable missing parent model after `.`".to_string()
                }
            },
            IncompleteKind::Section(section_kind) => match section_kind {
                SectionKind::MissingLabel => "section missing label".to_string(),
                SectionKind::MissingEndOfLine => {
                    "section must be followed by a new line".to_string()
                }
            },
            IncompleteKind::Parameter(parameter_kind) => match parameter_kind {
                ParameterKind::MissingIdentifier => "parameter missing identifier".to_string(),
                ParameterKind::MissingEqualsSign => "parameter missing `=`".to_string(),
                ParameterKind::MissingValue => "parameter missing value".to_string(),
                ParameterKind::MissingEndOfLine => {
                    "parameter must be followed by a new line".to_string()
                }
                ParameterKind::MissingUnit => "parameter missing unit after `:`".to_string(),
                ParameterKind::LimitMissingMin => "limit missing minimum value".to_string(),
                ParameterKind::LimitMissingComma => "limit missing `,`".to_string(),
                ParameterKind::LimitMissingMax => "limit missing maximum value".to_string(),
                ParameterKind::LimitMissingValues => "limit missing values".to_string(),
                ParameterKind::PiecewiseMissingExpr => "piecewise missing expression".to_string(),
                ParameterKind::PiecewiseMissingIf => "piecewise missing `if`".to_string(),
                ParameterKind::PiecewiseMissingIfExpr => {
                    "piecewise missing conditional expression after `if`".to_string()
                }
            },
            IncompleteKind::Test(test_kind) => match test_kind {
                TestKind::MissingColon => "test missing `:`".to_string(),
                TestKind::MissingExpr => "test missing expression".to_string(),
                TestKind::MissingEndOfLine => "test must be followed by a new line".to_string(),
                TestKind::MissingInputs => "test missing inputs".to_string(),
            },
            IncompleteKind::Unit(unit_kind) => match unit_kind {
                UnitKind::MissingSecondTerm { operator } => {
                    let operator_str = match operator {
                        oneil_ast::unit::UnitOp::Multiply => "*".to_string(),
                        oneil_ast::unit::UnitOp::Divide => "/".to_string(),
                    };
                    format!("operator `{}` missing second operand", operator_str)
                }
                UnitKind::MissingExponent => "unit missing exponent".to_string(),
                UnitKind::ParenMissingExpr => "parenthesized unit missing expression".to_string(),
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
