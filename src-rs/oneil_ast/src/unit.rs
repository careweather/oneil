//! Unit expression constructs for the AST
//!
//! This module contains structures for representing unit expressions in Oneil programs,
//! including unit operations, unit definitions, and unit exponents.

use crate::{naming::IdentifierNode, node::Node};

/// Represents a unit expression
#[derive(Debug, Clone, PartialEq)]
pub enum UnitExpr {
    /// Binary operation on unit expressions
    BinaryOp {
        /// The unit operator
        op: UnitOpNode,
        /// The left operand
        left: Box<UnitExprNode>,
        /// The right operand
        right: Box<UnitExprNode>,
    },
    /// Parenthesized unit expression
    Parenthesized {
        /// The expression inside parentheses
        expr: Box<UnitExprNode>,
    },
    /// A unit with optional exponent
    Unit {
        /// The unit identifier
        identifier: IdentifierNode,
        /// The optional exponent
        exponent: Option<UnitExponentNode>,
    },
}

/// A node containing a unit expression
pub type UnitExprNode = Node<UnitExpr>;

impl UnitExpr {
    /// Creates a binary operation unit expression
    pub fn binary_op(op: UnitOpNode, left: UnitExprNode, right: UnitExprNode) -> Self {
        let left = Box::new(left);
        let right = Box::new(right);
        Self::BinaryOp { op, left, right }
    }

    /// Creates a parenthesized unit expression
    pub fn parenthesized(expr: UnitExprNode) -> Self {
        Self::Parenthesized {
            expr: Box::new(expr),
        }
    }

    /// Creates a unit expression with optional exponent
    pub fn unit(identifier: IdentifierNode, exponent: Option<UnitExponentNode>) -> Self {
        Self::Unit {
            identifier,
            exponent,
        }
    }
}

/// Unit operators for unit expressions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnitOp {
    /// Multiplication operator for units (*)
    Multiply,
    /// Division operator for units (/)
    Divide,
}

/// A node containing a unit operator
pub type UnitOpNode = Node<UnitOp>;

impl UnitOp {
    /// Creates a multiplication operator for units
    pub fn multiply() -> Self {
        Self::Multiply
    }

    /// Creates a division operator for units
    pub fn divide() -> Self {
        Self::Divide
    }
}

/// A unit with an optional exponent
#[derive(Debug, Clone, PartialEq)]
pub struct Unit {
    identifier: IdentifierNode,
    exponent: Option<f64>,
}

/// A node containing a unit definition
pub type UnitNode = Node<Unit>;

/// A unit exponent value
///
/// Unit exponents specify the power to which a unit is raised
/// (e.g., m², kg³).
#[derive(Debug, Clone, PartialEq)]
pub struct UnitExponent(f64);

/// A node containing a unit exponent
pub type UnitExponentNode = Node<UnitExponent>;

impl UnitExponent {
    /// Creates a new unit exponent with the given value
    pub fn new(value: f64) -> Self {
        Self(value)
    }
}
