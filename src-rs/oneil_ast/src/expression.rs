//! Expression constructs for the AST
//!
//! This module contains structures for representing expressions in the Oneil language,
//! including binary operations, unary operations, function calls, variables, and literals.

use crate::{naming::IdentifierNode, node::Node};

/// An expression in the Oneil language
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Comparison operation with left and right operands
    ComparisonOp {
        /// The left operand
        left: ExprNode,
        /// The comparison operator
        op: ComparisonOpNode,
        /// The right operand
        right: ExprNode,
        /// Chained comparison operations (order matters)
        rest_chained: Vec<(ComparisonOpNode, ExprNode)>,
    },
    /// Binary operation with left and right operands
    BinaryOp {
        /// The binary operator
        op: BinaryOpNode,
        /// The left operand
        left: ExprNode,
        /// The right operand
        right: ExprNode,
    },

    /// Unary operation with a single operand
    UnaryOp {
        /// The unary operator
        op: UnaryOpNode,
        /// The operand expression
        expr: ExprNode,
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
        expr: ExprNode,
    },

    /// Variable reference
    Variable(VariableNode),

    /// Literal value
    Literal(LiteralNode),
}

/// A node containing an expression
pub type ExprNode = Node<Expr>;

impl Expr {
    /// Creates a comparison operation expression
    #[must_use]
    pub fn comparison_op(
        op: ComparisonOpNode,
        left: ExprNode,
        right: ExprNode,
        rest_chained: Vec<(ComparisonOpNode, ExprNode)>,
    ) -> Self {
        Self::ComparisonOp {
            op,
            left,
            right,
            rest_chained,
        }
    }

    /// Creates a binary operation expression
    #[must_use]
    pub fn binary_op(op: BinaryOpNode, left: ExprNode, right: ExprNode) -> Self {
        Self::BinaryOp { op, left, right }
    }

    /// Creates a unary operation expression
    #[must_use]
    pub fn unary_op(op: UnaryOpNode, expr: ExprNode) -> Self {
        Self::UnaryOp { op, expr }
    }

    /// Creates a function call expression
    #[must_use]
    pub const fn function_call(name: IdentifierNode, args: Vec<ExprNode>) -> Self {
        Self::FunctionCall { name, args }
    }

    /// Creates a parenthesized expression
    #[must_use]
    pub fn parenthesized(expr: ExprNode) -> Self {
        Self::Parenthesized { expr }
    }

    /// Creates a variable reference expression
    #[must_use]
    pub const fn variable(var: VariableNode) -> Self {
        Self::Variable(var)
    }

    /// Creates a literal expression
    #[must_use]
    pub const fn literal(lit: LiteralNode) -> Self {
        Self::Literal(lit)
    }
}

/// Comparison operators for expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonOp {
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
}

/// A node containing a comparison operator
pub type ComparisonOpNode = Node<ComparisonOp>;

impl ComparisonOp {
    /// Creates a less than operator
    #[must_use]
    pub const fn less_than() -> Self {
        Self::LessThan
    }

    /// Creates a less than or equal operator
    #[must_use]
    pub const fn less_than_eq() -> Self {
        Self::LessThanEq
    }

    /// Creates a greater than operator
    #[must_use]
    pub const fn greater_than() -> Self {
        Self::GreaterThan
    }

    /// Creates a greater than or equal operator
    #[must_use]
    pub const fn greater_than_eq() -> Self {
        Self::GreaterThanEq
    }

    /// Creates an equality operator
    #[must_use]
    pub const fn eq() -> Self {
        Self::Eq
    }

    /// Creates an inequality operator
    #[must_use]
    pub const fn not_eq() -> Self {
        Self::NotEq
    }
}

/// Binary operators for expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    #[must_use]
    pub const fn add() -> Self {
        Self::Add
    }

    /// Creates a subtraction operator
    #[must_use]
    pub const fn sub() -> Self {
        Self::Sub
    }

    /// Creates a true subtraction operator
    #[must_use]
    pub const fn true_sub() -> Self {
        Self::TrueSub
    }

    /// Creates a multiplication operator
    #[must_use]
    pub const fn mul() -> Self {
        Self::Mul
    }

    /// Creates a division operator
    #[must_use]
    pub const fn div() -> Self {
        Self::Div
    }

    /// Creates a true division operator
    #[must_use]
    pub const fn true_div() -> Self {
        Self::TrueDiv
    }

    /// Creates a modulo operator
    #[must_use]
    pub const fn modulo() -> Self {
        Self::Mod
    }

    /// Creates a power operator
    #[must_use]
    pub const fn pow() -> Self {
        Self::Pow
    }

    /// Creates a logical AND operator
    #[must_use]
    pub const fn and() -> Self {
        Self::And
    }

    /// Creates a logical OR operator
    #[must_use]
    pub const fn or() -> Self {
        Self::Or
    }

    /// Creates a min/max operator
    #[must_use]
    pub const fn min_max() -> Self {
        Self::MinMax
    }
}

/// Unary operators for expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    #[must_use]
    pub const fn neg() -> Self {
        Self::Neg
    }

    /// Creates a logical NOT operator
    #[must_use]
    pub const fn not() -> Self {
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
    #[must_use]
    pub const fn identifier(id: IdentifierNode) -> Self {
        Self::Identifier(id)
    }

    /// Creates an accessor variable reference
    #[must_use]
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
    #[must_use]
    pub const fn number(num: f64) -> Self {
        Self::Number(num)
    }

    /// Creates a string literal
    #[must_use]
    pub const fn string(str: String) -> Self {
        Self::String(str)
    }

    /// Creates a boolean literal
    #[must_use]
    pub const fn boolean(bool: bool) -> Self {
        Self::Boolean(bool)
    }
}
