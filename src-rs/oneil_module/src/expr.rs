use crate::reference::{Identifier, ModulePath};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    BinaryOp {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    UnaryOp {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    FunctionCall {
        name: FunctionName,
        args: Vec<Expr>,
    },
    Variable(Variable),
    Literal {
        value: Literal,
    },
}

impl Expr {
    pub fn binary_op(op: BinaryOp, left: Expr, right: Expr) -> Self {
        Self::BinaryOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    pub fn unary_op(op: UnaryOp, expr: Expr) -> Self {
        Self::UnaryOp {
            op,
            expr: Box::new(expr),
        }
    }

    pub fn function_call(name: FunctionName, args: Vec<Expr>) -> Self {
        Self::FunctionCall { name, args }
    }

    pub fn local_variable(ident: Identifier) -> Self {
        Self::Variable(Variable::Local(ident))
    }

    pub fn parameter_variable(ident: Identifier) -> Self {
        Self::Variable(Variable::Parameter(ident))
    }

    pub fn external_variable(module: ModulePath, ident: Identifier) -> Self {
        Self::Variable(Variable::External { module, ident })
    }

    pub fn literal(value: Literal) -> Self {
        Self::Literal { value }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    TrueSub,
    Mul,
    Div,
    TrueDiv,
    Mod,
    Pow,
    LessThan,
    LessThanEq,
    GreaterThan,
    GreaterThanEq,
    Eq,
    NotEq,
    And,
    Or,
    MinMax,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionName {
    Builtin(BuiltinFunction),
    Imported(String),
}

impl FunctionName {
    pub fn min() -> Self {
        Self::Builtin(BuiltinFunction::Min)
    }

    pub fn max() -> Self {
        Self::Builtin(BuiltinFunction::Max)
    }

    pub fn sin() -> Self {
        Self::Builtin(BuiltinFunction::Sin)
    }

    pub fn cos() -> Self {
        Self::Builtin(BuiltinFunction::Cos)
    }

    pub fn tan() -> Self {
        Self::Builtin(BuiltinFunction::Tan)
    }

    pub fn asin() -> Self {
        Self::Builtin(BuiltinFunction::Asin)
    }

    pub fn acos() -> Self {
        Self::Builtin(BuiltinFunction::Acos)
    }

    pub fn atan() -> Self {
        Self::Builtin(BuiltinFunction::Atan)
    }

    pub fn sqrt() -> Self {
        Self::Builtin(BuiltinFunction::Sqrt)
    }

    pub fn ln() -> Self {
        Self::Builtin(BuiltinFunction::Ln)
    }

    pub fn log() -> Self {
        Self::Builtin(BuiltinFunction::Log)
    }

    pub fn log10() -> Self {
        Self::Builtin(BuiltinFunction::Log10)
    }

    pub fn floor() -> Self {
        Self::Builtin(BuiltinFunction::Floor)
    }

    pub fn ceiling() -> Self {
        Self::Builtin(BuiltinFunction::Ceiling)
    }

    pub fn extent() -> Self {
        Self::Builtin(BuiltinFunction::Extent)
    }

    pub fn range() -> Self {
        Self::Builtin(BuiltinFunction::Range)
    }

    pub fn abs() -> Self {
        Self::Builtin(BuiltinFunction::Abs)
    }

    pub fn sign() -> Self {
        Self::Builtin(BuiltinFunction::Sign)
    }

    pub fn mid() -> Self {
        Self::Builtin(BuiltinFunction::Mid)
    }

    pub fn strip() -> Self {
        Self::Builtin(BuiltinFunction::Strip)
    }

    pub fn minmax() -> Self {
        Self::Builtin(BuiltinFunction::MinMax)
    }

    pub fn imported(name: String) -> Self {
        Self::Imported(name)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BuiltinFunction {
    Min,
    Max,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Sqrt,
    Ln,
    Log,
    Log10,
    Floor,
    Ceiling,
    Extent,
    Range,
    Abs,
    Sign,
    Mid,
    Strip,
    MinMax,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Variable {
    Local(Identifier),
    Parameter(Identifier),
    External {
        module: ModulePath,
        ident: Identifier,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
}

impl Literal {
    pub fn number(value: f64) -> Self {
        Self::Number(value)
    }

    pub fn string(value: String) -> Self {
        Self::String(value)
    }

    pub fn boolean(value: bool) -> Self {
        Self::Boolean(value)
    }
}
