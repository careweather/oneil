//! Expression system for mathematical and logical operations in Oneil.
//!
//! This module provides a rich expression language that supports mathematical
//! operations, function calls, variable references, and literals. The expression
//! system is designed to be extensible and type-safe, supporting both built-in
//! functions and imported Python functions.

use crate::{
    reference::{Identifier, ModelPath},
    span::{Span, WithSpan},
};

/// Abstract syntax tree for mathematical and logical expressions.
///
/// `Expr` represents the core expression language in Oneil, supporting:
///
/// - **Binary Operations**: Mathematical and logical operations on two operands
/// - **Unary Operations**: Operations on a single operand (negation, logical NOT)
/// - **Function Calls**: Built-in and imported function invocations
/// - **Variables**: References to local, parameter, and external variables
/// - **Literals**: Constant values (numbers, strings, booleans)
///
/// All expressions are immutable and can be easily cloned for manipulation.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
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
    ///
    /// let left = Expr::literal(Literal::number(5.0));
    /// let right = Expr::literal(Literal::number(3.0));
    /// let expr = Expr::binary_op(BinaryOp::Add, left, right);
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
    ///
    /// let operand = Expr::literal(Literal::number(5.0));
    /// let expr = Expr::unary_op(UnaryOp::Neg, operand);
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
    ///
    /// let args = vec![Expr::literal(Literal::number(3.14))];
    /// let expr = Expr::function_call(FunctionName::sin(), args);
    /// ```
    FunctionCall {
        /// The name of the function to call.
        name: WithSpan<FunctionName>,
        /// The arguments to pass to the function.
        args: Vec<ExprWithSpan>,
    },
    /// Variable reference (local, parameter, or external).
    Variable(WithSpan<Variable>),
    /// Constant literal value.
    Literal {
        /// The literal value.
        value: Literal,
    },
}

pub type ExprWithSpan = WithSpan<Expr>;

impl Expr {
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
    ///
    /// let left = Expr::literal(Literal::number(5.0));
    /// let right = Expr::literal(Literal::number(3.0));
    /// let expr = Expr::binary_op(BinaryOp::Add, left, right);
    /// ```
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
    ///
    /// let operand = Expr::literal(Literal::number(5.0));
    /// let expr = Expr::unary_op(UnaryOp::Neg, operand);
    /// ```
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
    /// use oneil_ir::expr::{Expr, FunctionName, Literal};
    ///
    /// let args = vec![Expr::literal(Literal::number(3.14))];
    /// let expr = Expr::function_call(FunctionName::sin(), args);
    /// ```
    pub fn function_call(name: WithSpan<FunctionName>, args: Vec<ExprWithSpan>) -> Self {
        Self::FunctionCall { name, args }
    }

    /// Creates a local variable reference.
    ///
    /// # Arguments
    ///
    /// * `ident` - The identifier of the local variable
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{expr::Expr, reference::Identifier};
    ///
    /// let expr = Expr::local_variable(Identifier::new("x"));
    /// ```
    pub fn local_variable(ident: Identifier, variable_span: Span) -> Self {
        Self::Variable(WithSpan::new(Variable::Local(ident), variable_span))
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
    pub fn parameter_variable(ident: Identifier, variable_span: Span) -> Self {
        Self::Variable(WithSpan::new(Variable::Parameter(ident), variable_span))
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
    pub fn external_variable(model: ModelPath, ident: Identifier, variable_span: Span) -> Self {
        Self::Variable(WithSpan::new(
            Variable::External { model, ident },
            variable_span,
        ))
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
    pub fn literal(value: Literal) -> Self {
        Self::Literal { value }
    }
}

/// Binary operators for mathematical and logical operations.
///
/// These operators are used in binary expressions to combine two operands.
/// The operators include standard arithmetic operations, comparison operators,
/// and logical operations.
#[derive(Debug, Clone, PartialEq)]
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
    /// Less than: `a < b`
    LessThan,
    /// Less than or equal: `a <= b`
    LessThanEq,
    /// Greater than: `a > b`
    GreaterThan,
    /// Greater than or equal: `a >= b`
    GreaterThanEq,
    /// Equality: `a == b`
    Eq,
    /// Inequality: `a != b`
    NotEq,
    /// Logical AND: `a && b`
    And,
    /// Logical OR: `a || b`
    Or,
    /// Minimum/maximum: `a | b`
    MinMax,
}

/// Unary operators for single-operand operations.
///
/// These operators are used in unary expressions to modify a single operand.
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
pub enum FunctionName {
    /// Built-in mathematical function.
    Builtin(BuiltinFunction),
    /// Function imported from a Python module.
    Imported(String),
}

impl FunctionName {
    /// Creates a reference to the `min` function.
    pub fn min() -> Self {
        Self::Builtin(BuiltinFunction::Min)
    }

    /// Creates a reference to the `max` function.
    pub fn max() -> Self {
        Self::Builtin(BuiltinFunction::Max)
    }

    /// Creates a reference to the `sin` function.
    pub fn sin() -> Self {
        Self::Builtin(BuiltinFunction::Sin)
    }

    /// Creates a reference to the `cos` function.
    pub fn cos() -> Self {
        Self::Builtin(BuiltinFunction::Cos)
    }

    /// Creates a reference to the `tan` function.
    pub fn tan() -> Self {
        Self::Builtin(BuiltinFunction::Tan)
    }

    /// Creates a reference to the `asin` function.
    pub fn asin() -> Self {
        Self::Builtin(BuiltinFunction::Asin)
    }

    /// Creates a reference to the `acos` function.
    pub fn acos() -> Self {
        Self::Builtin(BuiltinFunction::Acos)
    }

    /// Creates a reference to the `atan` function.
    pub fn atan() -> Self {
        Self::Builtin(BuiltinFunction::Atan)
    }

    /// Creates a reference to the `sqrt` function.
    pub fn sqrt() -> Self {
        Self::Builtin(BuiltinFunction::Sqrt)
    }

    /// Creates a reference to the `ln` function.
    pub fn ln() -> Self {
        Self::Builtin(BuiltinFunction::Ln)
    }

    /// Creates a reference to the `log` function.
    pub fn log() -> Self {
        Self::Builtin(BuiltinFunction::Log)
    }

    /// Creates a reference to the `log10` function.
    pub fn log10() -> Self {
        Self::Builtin(BuiltinFunction::Log10)
    }

    /// Creates a reference to the `floor` function.
    pub fn floor() -> Self {
        Self::Builtin(BuiltinFunction::Floor)
    }

    /// Creates a reference to the `ceiling` function.
    pub fn ceiling() -> Self {
        Self::Builtin(BuiltinFunction::Ceiling)
    }

    /// Creates a reference to the `extent` function.
    pub fn extent() -> Self {
        Self::Builtin(BuiltinFunction::Extent)
    }

    /// Creates a reference to the `range` function.
    pub fn range() -> Self {
        Self::Builtin(BuiltinFunction::Range)
    }

    /// Creates a reference to the `abs` function.
    pub fn abs() -> Self {
        Self::Builtin(BuiltinFunction::Abs)
    }

    /// Creates a reference to the `sign` function.
    pub fn sign() -> Self {
        Self::Builtin(BuiltinFunction::Sign)
    }

    /// Creates a reference to the `mid` function.
    pub fn mid() -> Self {
        Self::Builtin(BuiltinFunction::Mid)
    }

    /// Creates a reference to the `strip` function.
    pub fn strip() -> Self {
        Self::Builtin(BuiltinFunction::Strip)
    }

    /// Creates a reference to the `minmax` function.
    pub fn minmax() -> Self {
        Self::Builtin(BuiltinFunction::MinMax)
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
    /// use oneil_ir::expr::FunctionName;
    ///
    /// let func = FunctionName::imported("numpy.random.normal".to_string());
    /// ```
    pub fn imported(name: String) -> Self {
        Self::Imported(name)
    }
}

/// Built-in mathematical functions available in Oneil.
///
/// These functions provide common mathematical operations and are
/// always available without requiring imports.
#[derive(Debug, Clone, PartialEq)]
pub enum BuiltinFunction {
    /// Minimum of two values: `min(a, b)`
    Min,
    /// Maximum of two values: `max(a, b)`
    Max,
    /// Sine function: `sin(x)`
    Sin,
    /// Cosine function: `cos(x)`
    Cos,
    /// Tangent function: `tan(x)`
    Tan,
    /// Arcsine function: `asin(x)`
    Asin,
    /// Arccosine function: `acos(x)`
    Acos,
    /// Arctangent function: `atan(x)`
    Atan,
    /// Square root: `sqrt(x)`
    Sqrt,
    /// Natural logarithm: `ln(x)`
    Ln,
    /// Logarithm (base e): `log(x)`
    Log,
    /// Base-10 logarithm: `log10(x)`
    Log10,
    /// Floor function: `floor(x)`
    Floor,
    /// Ceiling function: `ceiling(x)`
    Ceiling,
    /// Extent function: `extent(x)`
    Extent,
    /// Range function: `range(x)`
    Range,
    /// Absolute value: `abs(x)`
    Abs,
    /// Sign function: `sign(x)`
    Sign,
    /// Midpoint function: `mid(x)`
    Mid,
    /// Strip function: `strip(x)`
    Strip,
    /// Min/max function: `mnmx(x)`
    MinMax,
}

/// Variable references in expressions.
///
/// Variables can refer to different scopes:
/// - **Local**: Variables defined in the current scope
/// - **Parameter**: Parameters defined in the current model
/// - **External**: Parameters defined in other models
#[derive(Debug, Clone, PartialEq)]
pub enum Variable {
    /// Local variable in the current scope.
    Local(Identifier),
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
    pub fn local(ident: Identifier) -> Self {
        Self::Local(ident)
    }

    pub fn parameter(ident: Identifier) -> Self {
        Self::Parameter(ident)
    }

    pub fn external(model: ModelPath, ident: Identifier) -> Self {
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
    pub fn number(value: f64) -> Self {
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
    pub fn string(value: String) -> Self {
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
    pub fn boolean(value: bool) -> Self {
        Self::Boolean(value)
    }
}
