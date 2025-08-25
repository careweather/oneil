//! Expression system for mathematical and logical operations in Oneil.
//!
//! This module provides a rich expression language that supports mathematical
//! operations, function calls, variable references, and literals. The expression
//! system is designed to be extensible and type-safe, supporting both built-in
//! functions and imported Python functions.

use crate::{
    reference::{Identifier, ModelPath},
    span::WithSpan,
};

/// Abstract syntax tree for mathematical and logical expressions.
///
/// `Expr` represents the core expression language in Oneil, supporting:
///
/// - **Comparison Operations**: Comparison operations with support for chaining
/// - **Binary Operations**: Mathematical and logical operations on two operands
/// - **Unary Operations**: Operations on a single operand (negation, logical NOT)
/// - **Function Calls**: Built-in and imported function invocations
/// - **Variables**: References to local, parameter, and external variables
/// - **Literals**: Constant values (numbers, strings, booleans)
///
/// All expressions are immutable and can be easily cloned for manipulation.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Comparison operation with left and right operands, supporting chaining.
    ///
    /// # Arguments
    ///
    /// * `op` - The comparison operator
    /// * `left` - The left-hand operand
    /// * `right` - The right-hand operand
    /// * `rest_chained` - Additional chained comparison operations (order matters)
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::expr::{Expr, ComparisonOp, Literal};
    /// use oneil_ir::span::WithSpan;
    ///
    /// let left = WithSpan::test_new(Expr::literal(Literal::number(1.0)));
    /// let middle = WithSpan::test_new(Expr::literal(Literal::number(2.0)));
    /// let right = WithSpan::test_new(Expr::literal(Literal::number(3.0)));
    /// let expr = Expr::comparison_op(
    ///     WithSpan::test_new(ComparisonOp::LessThan),
    ///     left,
    ///     middle,
    ///     vec![(WithSpan::test_new(ComparisonOp::LessThan), right)]
    /// );
    /// ```
    ComparisonOp {
        /// The comparison operator to apply.
        op: WithSpan<ComparisonOp>,
        /// The left-hand operand.
        left: Box<ExprWithSpan>,
        /// The right-hand operand.
        right: Box<ExprWithSpan>,
        /// Chained comparison operations (order matters).
        rest_chained: Vec<(WithSpan<ComparisonOp>, ExprWithSpan)>,
    },
    /// Binary operation combining two expressions with an operator.
    ///
    /// # Arguments
    ///
    /// * `op` - The binary operator
    /// * `left` - The left-hand operand
    /// * `right` - The right-hand operand
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::expr::{Expr, BinaryOp, Literal};
    /// use oneil_ir::span::WithSpan;
    ///
    /// let left = WithSpan::test_new(Expr::literal(Literal::number(5.0)));
    /// let right = WithSpan::test_new(Expr::literal(Literal::number(3.0)));
    /// let expr = Expr::binary_op(WithSpan::test_new(BinaryOp::Add), left, right);
    /// ```
    BinaryOp {
        /// The binary operator to apply.
        op: WithSpan<BinaryOp>,
        /// The left-hand operand.
        left: Box<ExprWithSpan>,
        /// The right-hand operand.
        right: Box<ExprWithSpan>,
    },
    /// Unary operation applied to a single expression.
    ///
    /// # Arguments
    ///
    /// * `op` - The unary operator
    /// * `expr` - The operand expression
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::expr::{Expr, UnaryOp, Literal};
    /// use oneil_ir::span::WithSpan;
    ///
    /// let operand = WithSpan::test_new(Expr::literal(Literal::number(5.0)));
    /// let expr = Expr::unary_op(WithSpan::test_new(UnaryOp::Neg), operand);
    /// ```
    UnaryOp {
        /// The unary operator to apply.
        op: WithSpan<UnaryOp>,
        /// The operand expression.
        expr: Box<ExprWithSpan>,
    },
    /// Function call with a name and argument list.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the function to call
    /// * `args` - The arguments to pass to the function
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::expr::{Expr, FunctionName, Literal};
    /// use oneil_ir::reference::Identifier;
    /// use oneil_ir::span::WithSpan;
    ///
    /// let args = vec![WithSpan::test_new(Expr::literal(Literal::number(3.14)))];
    /// let expr = Expr::function_call(WithSpan::test_new(FunctionName::imported(Identifier::new("foo"))), args);
    /// ```
    FunctionCall {
        /// The name of the function to call.
        name: WithSpan<FunctionName>,
        /// The arguments to pass to the function.
        args: Vec<ExprWithSpan>,
    },
    /// Variable reference (local, parameter, or external).
    Variable(Variable),
    /// Constant literal value.
    Literal {
        /// The literal value.
        value: Literal,
    },
}

/// An expression with associated source location information.
///
/// This type alias provides a convenient way to work with expressions
/// that include source location spans for error reporting and debugging.
pub type ExprWithSpan = WithSpan<Expr>;

impl Expr {
    /// Creates a comparison operation expression.
    ///
    /// # Arguments
    ///
    /// * `op` - The comparison operator
    /// * `left` - The left-hand operand
    /// * `right` - The right-hand operand
    /// * `rest_chained` - Additional chained comparison operations (order matters)
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::expr::{Expr, ComparisonOp, Literal};
    /// use oneil_ir::span::WithSpan;
    ///
    /// let left = WithSpan::test_new(Expr::literal(Literal::number(1.0)));
    /// let middle = WithSpan::test_new(Expr::literal(Literal::number(2.0)));
    /// let right = WithSpan::test_new(Expr::literal(Literal::number(3.0)));
    /// let expr = Expr::comparison_op(
    ///     WithSpan::test_new(ComparisonOp::LessThan),
    ///     left,
    ///     middle,
    ///     vec![(WithSpan::test_new(ComparisonOp::LessThan), right)]
    /// );
    /// ```
    #[must_use]
    pub fn comparison_op(
        op: WithSpan<ComparisonOp>,
        left: ExprWithSpan,
        right: ExprWithSpan,
        rest_chained: Vec<(WithSpan<ComparisonOp>, ExprWithSpan)>,
    ) -> Self {
        Self::ComparisonOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
            rest_chained,
        }
    }

    /// Creates a binary operation expression.
    ///
    /// # Arguments
    ///
    /// * `op` - The binary operator
    /// * `left` - The left-hand operand
    /// * `right` - The right-hand operand
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::expr::{Expr, BinaryOp, Literal};
    /// use oneil_ir::span::WithSpan;
    ///
    /// let left = WithSpan::test_new(Expr::literal(Literal::number(5.0)));
    /// let right = WithSpan::test_new(Expr::literal(Literal::number(3.0)));
    /// let expr = Expr::binary_op(WithSpan::test_new(BinaryOp::Add), left, right);
    /// ```
    #[must_use]
    pub fn binary_op(op: WithSpan<BinaryOp>, left: ExprWithSpan, right: ExprWithSpan) -> Self {
        Self::BinaryOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    /// Creates a unary operation expression.
    ///
    /// # Arguments
    ///
    /// * `op` - The unary operator
    /// * `expr` - The operand expression
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::expr::{Expr, UnaryOp, Literal};
    /// use oneil_ir::span::WithSpan;
    ///
    /// let operand = WithSpan::test_new(Expr::literal(Literal::number(5.0)));
    /// let expr = Expr::unary_op(WithSpan::test_new(UnaryOp::Neg), operand);
    /// ```
    #[must_use]
    pub fn unary_op(op: WithSpan<UnaryOp>, expr: ExprWithSpan) -> Self {
        Self::UnaryOp {
            op,
            expr: Box::new(expr),
        }
    }

    /// Creates a function call expression.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the function to call
    /// * `args` - The arguments to pass to the function
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{expr::{Expr, FunctionName, Literal}, reference::Identifier};
    /// use oneil_ir::span::WithSpan;
    ///
    /// let args = vec![WithSpan::test_new(Expr::literal(Literal::number(3.14)))];
    /// let expr = Expr::function_call(WithSpan::test_new(FunctionName::imported(Identifier::new("foo"))), args);
    /// ```
    #[must_use]
    pub const fn function_call(name: WithSpan<FunctionName>, args: Vec<ExprWithSpan>) -> Self {
        Self::FunctionCall { name, args }
    }

    /// Creates a built-in variable reference.
    ///
    /// # Arguments
    ///
    /// * `ident` - The identifier of the built-in variable
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{expr::Expr, reference::Identifier};
    ///
    /// let expr = Expr::builtin_variable(Identifier::new("pi"));
    /// ```
    #[must_use]
    pub const fn builtin_variable(ident: Identifier) -> Self {
        Self::Variable(Variable::Builtin(ident))
    }

    /// Creates a parameter variable reference.
    ///
    /// # Arguments
    ///
    /// * `ident` - The identifier of the parameter
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{expr::Expr, reference::Identifier};
    ///
    /// let expr = Expr::parameter_variable(Identifier::new("radius"));
    /// ```
    #[must_use]
    pub const fn parameter_variable(ident: Identifier) -> Self {
        Self::Variable(Variable::Parameter(ident))
    }

    /// Creates an external variable reference.
    ///
    /// # Arguments
    ///
    /// * `model` - The model path where the variable is defined
    /// * `ident` - The identifier of the variable in that model
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{expr::Expr, reference::{Identifier, ModelPath}};
    ///
    /// let expr = Expr::external_variable(
    ///     ModelPath::new("math"),
    ///     Identifier::new("pi")
    /// );
    /// ```
    #[must_use]
    pub const fn external_variable(model: ModelPath, ident: Identifier) -> Self {
        Self::Variable(Variable::External { model, ident })
    }

    /// Creates a literal expression.
    ///
    /// # Arguments
    ///
    /// * `value` - The literal value
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::expr::{Expr, Literal};
    ///
    /// let expr = Expr::literal(Literal::number(42.0));
    /// ```
    #[must_use]
    pub const fn literal(value: Literal) -> Self {
        Self::Literal { value }
    }
}

/// Binary operators for mathematical and logical operations.
///
/// These operators are used in binary expressions to combine two operands.
/// The operators include standard arithmetic operations, comparison operators,
/// and logical operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    /// Addition: `a + b`
    Add,
    /// Subtraction: `a - b`
    Sub,
    /// True subtraction: `a -- b`
    TrueSub,
    /// Multiplication: `a * b`
    Mul,
    /// Division: `a / b`
    Div,
    /// True division: `a // b`
    TrueDiv,
    /// Modulo: `a % b`
    Mod,
    /// Exponentiation: `a ^ b`
    Pow,
    /// Logical AND: `a && b`
    And,
    /// Logical OR: `a || b`
    Or,
    /// Minimum/maximum: `a | b`
    MinMax,
}

/// Comparison operators for expressions.
///
/// These operators are used in comparison expressions to compare two operands.
/// Comparison operations support chaining for expressions like `a < b < c`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonOp {
    /// Less than comparison: `a < b`
    LessThan,
    /// Less than or equal comparison: `a <= b`
    LessThanEq,
    /// Greater than comparison: `a > b`
    GreaterThan,
    /// Greater than or equal comparison: `a >= b`
    GreaterThanEq,
    /// Equality comparison: `a == b`
    Eq,
    /// Inequality comparison: `a != b`
    NotEq,
}

impl ComparisonOp {
    /// Creates a less than operator.
    #[must_use]
    pub const fn less_than() -> Self {
        Self::LessThan
    }

    /// Creates a less than or equal operator.
    #[must_use]
    pub const fn less_than_eq() -> Self {
        Self::LessThanEq
    }

    /// Creates a greater than operator.
    #[must_use]
    pub const fn greater_than() -> Self {
        Self::GreaterThan
    }

    /// Creates a greater than or equal operator.
    #[must_use]
    pub const fn greater_than_eq() -> Self {
        Self::GreaterThanEq
    }

    /// Creates an equality operator.
    #[must_use]
    pub const fn eq() -> Self {
        Self::Eq
    }

    /// Creates an inequality operator.
    #[must_use]
    pub const fn not_eq() -> Self {
        Self::NotEq
    }
}

/// Unary operators for single-operand operations.
///
/// These operators are used in unary expressions to modify a single operand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    /// Negation: `-a`
    Neg,
    /// Logical NOT: `!a`
    Not,
}

/// Function names for built-in and imported functions.
///
/// Functions in Oneil can be either built-in mathematical functions
/// or imported from Python modules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionName {
    /// Built-in mathematical function.
    Builtin(Identifier),
    /// Function imported from a Python module.
    Imported(Identifier),
}

impl FunctionName {
    /// Creates a reference to a built-in function.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the built-in function
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{expr::FunctionName, reference::Identifier};
    ///
    /// let func = FunctionName::builtin(Identifier::new("sin"));
    /// ```
    #[must_use]
    pub const fn builtin(name: Identifier) -> Self {
        Self::Builtin(name)
    }

    /// Creates a reference to an imported Python function.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the imported function
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{expr::FunctionName, reference::Identifier};
    ///
    /// let func = FunctionName::imported(Identifier::new("numpy.random.normal"));
    /// ```
    #[must_use]
    pub const fn imported(name: Identifier) -> Self {
        Self::Imported(name)
    }
}

/// Variable references in expressions.
///
/// Variables can refer to different scopes:
/// - **Local**: Variables defined in the current scope
/// - **Parameter**: Parameters defined in the current model
/// - **External**: Parameters defined in other models
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Variable {
    /// Built-in variable
    Builtin(Identifier),
    /// Parameter defined in the current model.
    Parameter(Identifier),
    /// Parameter defined in another model.
    External {
        /// The model where the parameter is defined.
        model: ModelPath,
        /// The identifier of the parameter in that model.
        ident: Identifier,
    },
}

impl Variable {
    /// Creates a built-in variable reference.
    ///
    /// # Arguments
    ///
    /// * `ident` - The identifier of the built-in variable
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{expr::Variable, reference::Identifier};
    ///
    /// let var = Variable::builtin(Identifier::new("x"));
    /// ```
    #[must_use]
    pub const fn builtin(ident: Identifier) -> Self {
        Self::Builtin(ident)
    }

    /// Creates a parameter variable reference.
    ///
    /// # Arguments
    ///
    /// * `ident` - The identifier of the parameter
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{expr::Variable, reference::Identifier};
    ///
    /// let var = Variable::parameter(Identifier::new("radius"));
    /// ```
    #[must_use]
    pub const fn parameter(ident: Identifier) -> Self {
        Self::Parameter(ident)
    }

    /// Creates an external variable reference.
    ///
    /// # Arguments
    ///
    /// * `model` - The model path where the variable is defined
    /// * `ident` - The identifier of the variable in that model
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{expr::Variable, reference::{Identifier, ModelPath}};
    ///
    /// let var = Variable::external(ModelPath::new("submodel"), Identifier::new("area"));
    /// ```
    #[must_use]
    pub const fn external(model: ModelPath, ident: Identifier) -> Self {
        Self::External { model, ident }
    }
}

/// Literal values that can appear in expressions.
///
/// Literals represent constant values and include numbers,
/// strings, and boolean values.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// Numeric literal (floating-point).
    Number(f64),
    /// String literal.
    String(String),
    /// Boolean literal.
    Boolean(bool),
}

impl Literal {
    /// Creates a numeric literal.
    ///
    /// # Arguments
    ///
    /// * `value` - The numeric value
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::expr::Literal;
    ///
    /// let lit = Literal::number(3.14159);
    /// ```
    #[must_use]
    pub const fn number(value: f64) -> Self {
        Self::Number(value)
    }

    /// Creates a string literal.
    ///
    /// # Arguments
    ///
    /// * `value` - The string value
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::expr::Literal;
    ///
    /// let lit = Literal::string("hello".to_string());
    /// ```
    #[must_use]
    pub const fn string(value: String) -> Self {
        Self::String(value)
    }

    /// Creates a boolean literal.
    ///
    /// # Arguments
    ///
    /// * `value` - The boolean value
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::expr::Literal;
    ///
    /// let lit = Literal::boolean(true);
    /// ```
    #[must_use]
    pub const fn boolean(value: bool) -> Self {
        Self::Boolean(value)
    }
}
