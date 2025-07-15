use crate::{naming::IdentifierNode, node::Node};

/// An expression in the Oneil language
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    BinaryOp {
        op: BinaryOpNode,
        left: Box<ExprNode>,
        right: Box<ExprNode>,
    },

    UnaryOp {
        op: UnaryOpNode,
        expr: Box<ExprNode>,
    },

    FunctionCall {
        name: IdentifierNode,
        args: Vec<ExprNode>,
    },

    Parenthesized {
        expr: Box<ExprNode>,
    },

    Variable(VariableNode),

    Literal(LiteralNode),
}

pub type ExprNode = Node<Expr>;

impl Expr {
    pub fn binary_op(op: BinaryOpNode, left: ExprNode, right: ExprNode) -> Self {
        Self::BinaryOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    pub fn unary_op(op: UnaryOpNode, expr: ExprNode) -> Self {
        Self::UnaryOp {
            op,
            expr: Box::new(expr),
        }
    }

    pub fn function_call(name: IdentifierNode, args: Vec<ExprNode>) -> Self {
        Self::FunctionCall { name, args }
    }

    pub fn parenthesized(expr: ExprNode) -> Self {
        Self::Parenthesized {
            expr: Box::new(expr),
        }
    }

    pub fn variable(var: VariableNode) -> Self {
        Self::Variable(var)
    }

    pub fn literal(lit: LiteralNode) -> Self {
        Self::Literal(lit)
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
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

pub type BinaryOpNode = Node<BinaryOp>;

impl BinaryOp {
    pub fn add() -> Self {
        Self::Add
    }

    pub fn sub() -> Self {
        Self::Sub
    }

    pub fn true_sub() -> Self {
        Self::TrueSub
    }

    pub fn mul() -> Self {
        Self::Mul
    }

    pub fn div() -> Self {
        Self::Div
    }

    pub fn true_div() -> Self {
        Self::TrueDiv
    }

    pub fn modulo() -> Self {
        Self::Mod
    }

    pub fn pow() -> Self {
        Self::Pow
    }

    pub fn less_than() -> Self {
        Self::LessThan
    }

    pub fn less_than_eq() -> Self {
        Self::LessThanEq
    }

    pub fn greater_than() -> Self {
        Self::GreaterThan
    }

    pub fn greater_than_eq() -> Self {
        Self::GreaterThanEq
    }

    pub fn eq() -> Self {
        Self::Eq
    }

    pub fn not_eq() -> Self {
        Self::NotEq
    }

    pub fn and() -> Self {
        Self::And
    }

    pub fn or() -> Self {
        Self::Or
    }

    pub fn min_max() -> Self {
        Self::MinMax
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum UnaryOp {
    Neg,
    Not,
}

pub type UnaryOpNode = Node<UnaryOp>;

impl UnaryOp {
    pub fn neg() -> Self {
        Self::Neg
    }

    pub fn not() -> Self {
        Self::Not
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Variable {
    Identifier(IdentifierNode),
    Accessor {
        parent: IdentifierNode,
        component: Box<VariableNode>,
    },
}

pub type VariableNode = Node<Variable>;

impl Variable {
    pub fn identifier(id: IdentifierNode) -> Self {
        Self::Identifier(id)
    }

    pub fn accessor(parent: IdentifierNode, component: VariableNode) -> Self {
        Self::Accessor {
            parent,
            component: Box::new(component),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
}

pub type LiteralNode = Node<Literal>;

impl Literal {
    pub fn number(num: f64) -> Self {
        Self::Number(num)
    }

    pub fn string(str: String) -> Self {
        Self::String(str)
    }

    pub fn boolean(bool: bool) -> Self {
        Self::Boolean(bool)
    }
}
