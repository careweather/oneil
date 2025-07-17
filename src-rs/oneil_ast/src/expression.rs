//! Expression constructs for the AST
//!
//! This module contains structures for representing expressions in the Oneil language,
//! including binary operations, unary operations, function calls, variables, and literals.

use crate::{naming::IdentifierNode, node::Node};

/// An expression in the Oneil language
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Binary operation with left and right operands
    BinaryOp {
        /// The binary operator
        op: BinaryOpNode,
        /// The left operand
        left: Box<ExprNode>,
        /// The right operand
        right: Box<ExprNode>,
    },

    /// Unary operation with a single operand
    UnaryOp {
        /// The unary operator
        op: UnaryOpNode,
        /// The operand expression
        expr: Box<ExprNode>,
    },

    /// Function call with arguments
    FunctionCall {
        /// The function name
        name: IdentifierNode,
        /// The function arguments
        args: Vec<ExprNode>,
    },

    /// Parenthesized expression
    Parenthesized {
        /// The expression inside parentheses
        expr: Box<ExprNode>,
    },

    /// Variable reference
    Variable(VariableNode),

    /// Literal value
    Literal(LiteralNode),
}

/// A node containing an expression
pub type ExprNode = Node<Expr>;

impl Expr {
    /// Creates a binary operation expression
    pub fn binary_op(op: BinaryOpNode, left: ExprNode, right: ExprNode) -> Self {
        Self::BinaryOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    /// Creates a unary operation expression
    pub fn unary_op(op: UnaryOpNode, expr: ExprNode) -> Self {
        Self::UnaryOp {
            op,
            expr: Box::new(expr),
        }
    }

    /// Creates a function call expression
    pub fn function_call(name: IdentifierNode, args: Vec<ExprNode>) -> Self {
        Self::FunctionCall { name, args }
    }

    /// Creates a parenthesized expression
    pub fn parenthesized(expr: ExprNode) -> Self {
        Self::Parenthesized {
            expr: Box::new(expr),
        }
    }

    /// Creates a variable reference expression
    pub fn variable(var: VariableNode) -> Self {
        Self::Variable(var)
    }

    /// Creates a literal expression
    pub fn literal(lit: LiteralNode) -> Self {
        Self::Literal(lit)
    }
}

/// Binary operators for expressions
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum BinaryOp {
    /// Addition operator (+)
    Add,
    /// Subtraction operator (-)
    Sub,
    /// True subtraction operator (--)
    TrueSub,
    /// Multiplication operator (*)
    Mul,
    /// Division operator (/)
    Div,
    /// True division operator (//)
    TrueDiv,
    /// Modulo operator (%)
    Mod,
    /// Power operator (**)
    Pow,
    /// Less than comparison (<)
    LessThan,
    /// Less than or equal comparison (<=)
    LessThanEq,
    /// Greater than comparison (>)
    GreaterThan,
    /// Greater than or equal comparison (>=)
    GreaterThanEq,
    /// Equality comparison (==)
    Eq,
    /// Inequality comparison (!=)
    NotEq,
    /// Logical AND operator (&&)
    And,
    /// Logical OR operator (||)
    Or,
    /// Min/max operator (min/max)
    MinMax,
}

/// A node containing a binary operator
pub type BinaryOpNode = Node<BinaryOp>;

impl BinaryOp {
    /// Creates an addition operator
    pub fn add() -> Self {
        Self::Add
    }

    /// Creates a subtraction operator
    pub fn sub() -> Self {
        Self::Sub
    }

    /// Creates a true subtraction operator
    pub fn true_sub() -> Self {
        Self::TrueSub
    }

    /// Creates a multiplication operator
    pub fn mul() -> Self {
        Self::Mul
    }

    /// Creates a division operator
    pub fn div() -> Self {
        Self::Div
    }

    /// Creates a true division operator
    pub fn true_div() -> Self {
        Self::TrueDiv
    }

    /// Creates a modulo operator
    pub fn modulo() -> Self {
        Self::Mod
    }

    /// Creates a power operator
    pub fn pow() -> Self {
        Self::Pow
    }

    /// Creates a less than operator
    pub fn less_than() -> Self {
        Self::LessThan
    }

    /// Creates a less than or equal operator
    pub fn less_than_eq() -> Self {
        Self::LessThanEq
    }

    /// Creates a greater than operator
    pub fn greater_than() -> Self {
        Self::GreaterThan
    }

    /// Creates a greater than or equal operator
    pub fn greater_than_eq() -> Self {
        Self::GreaterThanEq
    }

    /// Creates an equality operator
    pub fn eq() -> Self {
        Self::Eq
    }

    /// Creates an inequality operator
    pub fn not_eq() -> Self {
        Self::NotEq
    }

    /// Creates a logical AND operator
    pub fn and() -> Self {
        Self::And
    }

    /// Creates a logical OR operator
    pub fn or() -> Self {
        Self::Or
    }

    /// Creates a min/max operator
    pub fn min_max() -> Self {
        Self::MinMax
    }
}

/// Unary operators for expressions
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum UnaryOp {
    /// Negation operator (-)
    Neg,
    /// Logical NOT operator (!)
    Not,
}

/// A node containing a unary operator
pub type UnaryOpNode = Node<UnaryOp>;

impl UnaryOp {
    /// Creates a negation operator
    pub fn neg() -> Self {
        Self::Neg
    }

    /// Creates a logical NOT operator
    pub fn not() -> Self {
        Self::Not
    }
}

/// Variable references in expressions
#[derive(Debug, Clone, PartialEq)]
pub enum Variable {
    /// Simple identifier reference
    Identifier(IdentifierNode),
    /// Accessor pattern for nested references (e.g., parent.component)
    Accessor {
        /// The parent identifier
        parent: IdentifierNode,
        /// The component being accessed
        component: Box<VariableNode>,
    },
}

/// A node containing a variable reference
pub type VariableNode = Node<Variable>;

impl Variable {
    /// Creates a simple identifier variable reference
    pub fn identifier(id: IdentifierNode) -> Self {
        Self::Identifier(id)
    }

    /// Creates an accessor variable reference
    pub fn accessor(parent: IdentifierNode, component: VariableNode) -> Self {
        Self::Accessor {
            parent,
            component: Box::new(component),
        }
    }
}

/// Literal values in expressions
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// Numeric literal value
    Number(f64),
    /// String literal value
    String(String),
    /// Boolean literal value
    Boolean(bool),
}

/// A node containing a literal value
pub type LiteralNode = Node<Literal>;

impl Literal {
    /// Creates a numeric literal
    pub fn number(num: f64) -> Self {
        Self::Number(num)
    }

    /// Creates a string literal
    pub fn string(str: String) -> Self {
        Self::String(str)
    }

    /// Creates a boolean literal
    pub fn boolean(bool: bool) -> Self {
        Self::Boolean(bool)
    }
}
