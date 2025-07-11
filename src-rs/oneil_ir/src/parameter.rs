//! Parameter definitions and management for Oneil model IR.
//!
//! This module provides the data structures for representing parameters in Oneil,
//! including their values, dependencies, constraints, and metadata. Parameters
//! are the primary way to define configurable values in Oneil models.

use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

use crate::{debug_info::TraceLevel, expr::Expr, reference::Identifier, unit::CompositeUnit};

/// A collection of parameters that can be managed together.
///
/// `ParameterCollection` provides a container for multiple parameters,
/// allowing easy lookup and iteration over all parameters in a model.
/// It implements `Deref` to provide direct access to the underlying
/// parameter mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct ParameterCollection {
    parameters: HashMap<Identifier, Parameter>,
}

impl ParameterCollection {
    /// Creates a new parameter collection from a mapping of identifiers to parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - Mapping of parameter identifiers to their definitions
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{parameter::{ParameterCollection, Parameter, ParameterValue, Limits}, expr::{Expr, Literal}, reference::Identifier, debug_info::TraceLevel};
    /// use std::collections::HashMap;
    ///
    /// let mut params = HashMap::new();
    /// let param = Parameter::new(
    ///     std::collections::HashSet::new(),
    ///     Identifier::new("radius"),
    ///     ParameterValue::simple(Expr::literal(Literal::number(5.0)), None),
    ///     Limits::default(),
    ///     false,
    ///     TraceLevel::None,
    /// );
    /// params.insert(Identifier::new("radius"), param);
    ///
    /// let collection = ParameterCollection::new(params);
    /// ```
    pub fn new(parameters: HashMap<Identifier, Parameter>) -> Self {
        Self { parameters }
    }

    // TODO: add methods for getting performance parameters, evaluation order, etc.
}

impl Deref for ParameterCollection {
    type Target = HashMap<Identifier, Parameter>;

    fn deref(&self) -> &Self::Target {
        &self.parameters
    }
}

/// Represents a single parameter in an Oneil model.
///
/// Parameters are the primary mechanism for defining configurable values
/// in Oneil models. Each parameter has:
///
/// - **Dependencies**: Other parameters this parameter depends on
/// - **Value**: The expression or piecewise function that defines the parameter's value
/// - **Limits**: Constraints on valid values (continuous ranges or discrete sets)
/// - **Metadata**: Performance flags and trace levels for debugging
///
/// Parameters are immutable by design and support dependency analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    dependencies: HashSet<Identifier>,
    ident: Identifier,
    value: ParameterValue,
    limits: Limits,
    is_performance: bool,
    trace_level: TraceLevel,
}

impl Parameter {
    /// Creates a new parameter with the specified properties.
    ///
    /// # Arguments
    ///
    /// * `dependencies` - Set of parameter identifiers this parameter depends on
    /// * `ident` - The identifier for this parameter
    /// * `value` - The value expression or piecewise function
    /// * `limits` - Constraints on valid values
    /// * `is_performance` - Whether this is a performance parameter
    /// * `trace_level` - Trace level for debugging
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{parameter::{Parameter, ParameterValue, Limits}, expr::{Expr, Literal}, reference::Identifier, debug_info::TraceLevel};
    /// use std::collections::HashSet;
    ///
    /// let param = Parameter::new(
    ///     HashSet::new(),
    ///     Identifier::new("area"),
    ///     ParameterValue::simple(Expr::literal(Literal::number(25.0)), None),
    ///     Limits::default(),
    ///     false,
    ///     TraceLevel::None,
    /// );
    /// ```
    pub fn new(
        dependencies: HashSet<Identifier>,
        ident: Identifier,
        value: ParameterValue,
        limits: Limits,
        is_performance: bool,
        trace_level: TraceLevel,
    ) -> Self {
        Self {
            dependencies,
            ident,
            value,
            limits,
            is_performance,
            trace_level,
        }
    }

    /// Returns a reference to the identifier for this parameter.
    ///
    /// # Returns
    ///
    /// A reference to the identifier for this parameter.
    ///
    pub fn identifier(&self) -> &Identifier {
        &self.ident
    }

    /// Returns a reference to the set of parameter dependencies.
    ///
    /// Dependencies are the identifiers of other parameters that this parameter
    /// depends on for its value calculation. This information is crucial for
    /// determining evaluation order and detecting circular dependencies.
    ///
    /// # Returns
    ///
    /// A reference to the set of parameter identifiers this parameter depends on.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{parameter::{Parameter, ParameterValue, Limits}, expr::{Expr, Literal}, reference::Identifier, debug_info::TraceLevel};
    /// use std::collections::HashSet;
    ///
    /// let mut deps = HashSet::new();
    /// deps.insert(Identifier::new("radius"));
    ///
    /// let param = Parameter::new(
    ///     deps,
    ///     Identifier::new("area"),
    ///     ParameterValue::simple(Expr::literal(Literal::number(25.0)), None),
    ///     Limits::default(),
    ///     false,
    ///     TraceLevel::None,
    /// );
    ///
    /// assert!(param.dependencies().contains(&Identifier::new("radius")));
    /// ```
    pub fn dependencies(&self) -> &HashSet<Identifier> {
        &self.dependencies
    }

    /// Returns a reference to the value of this parameter.
    ///
    /// # Returns
    ///
    /// A reference to the ParameterValue containing either a simple expression or piecewise function.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{parameter::{Parameter, ParameterValue, Limits}, expr::{Expr, Literal}, reference::Identifier, debug_info::TraceLevel};
    /// use std::collections::HashSet;
    ///
    /// let param = Parameter::new(
    ///     HashSet::new(),
    ///     Identifier::new("x"),
    ///     ParameterValue::simple(Expr::literal(Literal::number(42.0)), None),
    ///     Limits::default(),
    ///     false,
    ///     TraceLevel::None,
    /// );
    ///
    /// match param.value() {
    ///     ParameterValue::Simple(expr, _) => {
    ///         assert_eq!(expr, &Expr::literal(Literal::number(42.0)));
    ///     },
    ///     _ => panic!("Expected simple value")
    /// }
    /// ```
    pub fn value(&self) -> &ParameterValue {
        &self.value
    }

    /// Returns a reference to the limits for this parameter.
    ///
    /// # Returns
    ///
    /// A reference to the Limits struct containing min/max bounds.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{parameter::{Parameter, ParameterValue, Limits}, expr::{Expr, Literal}, reference::Identifier, debug_info::TraceLevel};
    /// use std::collections::HashSet;
    ///
    /// let limits = Limits::continuous(Expr::literal(Literal::number(0.0)), Expr::literal(Literal::number(100.0)));
    /// let param = Parameter::new(
    ///     HashSet::new(),
    ///     Identifier::new("x"),
    ///     ParameterValue::simple(Expr::literal(Literal::number(42.0)), None),
    ///     limits.clone(),
    ///     false,
    ///     TraceLevel::None,
    /// );
    ///
    /// assert_eq!(param.limits(), &limits);
    /// ```
    pub fn limits(&self) -> &Limits {
        &self.limits
    }

    /// Returns whether this parameter is marked as a performance parameter.
    ///
    /// # Returns
    ///
    /// A boolean indicating if this is a performance parameter.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{parameter::{Parameter, ParameterValue, Limits}, expr::{Expr, Literal}, reference::Identifier, debug_info::TraceLevel};
    /// use std::collections::HashSet;
    ///
    /// let param = Parameter::new(
    ///     HashSet::new(),
    ///     Identifier::new("x"),
    ///     ParameterValue::simple(Expr::literal(Literal::number(42.0)), None),
    ///     Limits::default(),
    ///     true,
    ///     TraceLevel::None,
    /// );
    ///
    /// assert!(param.is_performance());
    /// ```
    pub fn is_performance(&self) -> bool {
        self.is_performance
    }

    /// Returns the trace level for this parameter.
    ///
    /// # Returns
    ///
    /// A reference to the TraceLevel enum indicating debug trace settings.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{parameter::{Parameter, ParameterValue, Limits}, expr::{Expr, Literal}, reference::Identifier, debug_info::TraceLevel};
    /// use std::collections::HashSet;
    ///
    /// let param = Parameter::new(
    ///     HashSet::new(),
    ///     Identifier::new("x"),
    ///     ParameterValue::simple(Expr::literal(Literal::number(42.0)), None),
    ///     Limits::default(),
    ///     false,
    ///     TraceLevel::Debug,
    /// );
    ///
    /// assert_eq!(param.trace_level(), &TraceLevel::Debug);
    /// ```
    pub fn trace_level(&self) -> &TraceLevel {
        &self.trace_level
    }

    /// Returns a reference to the optional unit of this parameter.
    ///
    /// # Returns
    ///
    /// A reference to the optional CompositeUnit for this parameter.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{parameter::{Parameter, ParameterValue, Limits}, expr::{Expr, Literal}, reference::Identifier, debug_info::TraceLevel, unit::{CompositeUnit, Unit}};
    /// use std::collections::HashSet;
    ///
    /// let units = vec![
    ///     Unit::new("m".to_string(), 2.0),  // meters squared
    ///     Unit::new("kg".to_string(), 1.0), // kilograms
    ///     Unit::new("s".to_string(), -2.0), // per second squared
    /// ];
    /// let composite = CompositeUnit::new(units);
    ///
    /// let param = Parameter::new(
    ///     HashSet::new(),
    ///     Identifier::new("length"),
    ///     ParameterValue::simple(Expr::literal(Literal::number(10.0)), Some(composite.clone())),
    ///     Limits::default(),
    ///     false,
    ///     TraceLevel::None,
    /// );
    ///
    /// assert_eq!(param.unit(), Some(&composite));
    /// ```
    pub fn unit(&self) -> Option<&CompositeUnit> {
        match &self.value {
            ParameterValue::Simple(_, unit) | ParameterValue::Piecewise(_, unit) => unit.as_ref(),
        }
    }
}

/// The value of a parameter, which can be either a simple expression or a piecewise function.
///
/// Parameter values support both simple expressions (like `2 * pi * radius`) and
/// piecewise functions that define different values based on conditions.
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterValue {
    /// A simple expression with an optional unit.
    Simple(Expr, Option<CompositeUnit>),
    /// A piecewise function with multiple expressions and conditions.
    Piecewise(Vec<PiecewiseExpr>, Option<CompositeUnit>),
}

impl ParameterValue {
    /// Creates a simple parameter value from an expression and optional unit.
    ///
    /// # Arguments
    ///
    /// * `expr` - The expression that defines the parameter's value
    /// * `unit` - Optional unit for the parameter value
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{parameter::ParameterValue, expr::{Expr, Literal}, unit::CompositeUnit};
    ///
    /// let expr = Expr::literal(Literal::number(3.14159));
    /// let value = ParameterValue::simple(expr, None);
    /// ```
    pub fn simple(expr: Expr, unit: Option<CompositeUnit>) -> Self {
        Self::Simple(expr, unit)
    }

    /// Creates a piecewise parameter value from a list of expressions and conditions.
    ///
    /// # Arguments
    ///
    /// * `exprs` - Vector of piecewise expressions with their conditions
    /// * `unit` - Optional unit for the parameter value
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{parameter::{ParameterValue, PiecewiseExpr}, expr::{Expr, Literal, BinaryOp}, reference::Identifier};
    ///
    /// let condition = Expr::binary_op(
    ///     BinaryOp::LessThan,
    ///     Expr::parameter_variable(Identifier::new("x")),
    ///     Expr::literal(Literal::number(0.0))
    /// );
    /// let expr = Expr::literal(Literal::number(-1.0));
    /// let piecewise = PiecewiseExpr::new(expr, condition);
    ///
    /// let value = ParameterValue::piecewise(vec![piecewise], None);
    /// ```
    pub fn piecewise(exprs: Vec<PiecewiseExpr>, unit: Option<CompositeUnit>) -> Self {
        Self::Piecewise(exprs, unit)
    }
}

/// A single expression in a piecewise function with its associated condition.
///
/// Piecewise expressions define a value and the condition under which that
/// value should be used. The condition is evaluated as a boolean expression.
#[derive(Debug, Clone, PartialEq)]
pub struct PiecewiseExpr {
    expr: Expr,
    if_expr: Expr,
}

impl PiecewiseExpr {
    /// Creates a new piecewise expression with a value and condition.
    ///
    /// # Arguments
    ///
    /// * `expr` - The expression that defines the value
    /// * `if_expr` - The condition expression (should evaluate to a boolean)
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{parameter::PiecewiseExpr, expr::{Expr, Literal, BinaryOp}, reference::Identifier};
    ///
    /// let value = Expr::literal(Literal::number(1.0));
    /// let condition = Expr::binary_op(
    ///     BinaryOp::GreaterThan,
    ///     Expr::parameter_variable(Identifier::new("x")),
    ///     Expr::literal(Literal::number(0.0))
    /// );
    ///
    /// let piecewise = PiecewiseExpr::new(value, condition);
    /// ```
    pub fn new(expr: Expr, if_expr: Expr) -> Self {
        Self { expr, if_expr }
    }
}

/// Constraints on valid parameter values.
///
/// Limits define the valid range or set of values that a parameter can take.
/// This is useful for validation and optimization purposes.
#[derive(Debug, Clone, PartialEq)]
pub enum Limits {
    /// No constraints on parameter values.
    Default,
    /// Continuous range with minimum and maximum values.
    Continuous {
        /// The minimum allowed value expression.
        min: Expr,
        /// The maximum allowed value expression.
        max: Expr,
    },
    /// Discrete set of allowed values.
    Discrete {
        /// Vector of expressions representing allowed values.
        values: Vec<Expr>,
    },
}

impl Limits {
    /// Creates default limits (no constraints).
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::parameter::Limits;
    ///
    /// let limits = Limits::default();
    /// ```
    pub fn default() -> Self {
        Self::Default
    }

    /// Creates continuous limits with minimum and maximum expressions.
    ///
    /// # Arguments
    ///
    /// * `min` - Expression for the minimum allowed value
    /// * `max` - Expression for the maximum allowed value
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{parameter::Limits, expr::{Expr, Literal}};
    ///
    /// let min = Expr::literal(Literal::number(0.0));
    /// let max = Expr::literal(Literal::number(100.0));
    /// let limits = Limits::continuous(min, max);
    /// ```
    pub fn continuous(min: Expr, max: Expr) -> Self {
        Self::Continuous { min, max }
    }

    /// Creates discrete limits with a set of allowed values.
    ///
    /// # Arguments
    ///
    /// * `values` - Vector of expressions representing allowed values
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{parameter::Limits, expr::{Expr, Literal}};
    ///
    /// let values = vec![
    ///     Expr::literal(Literal::number(1.0)),
    ///     Expr::literal(Literal::number(2.0)),
    ///     Expr::literal(Literal::number(3.0)),
    /// ];
    /// let limits = Limits::discrete(values);
    /// ```
    pub fn discrete(values: Vec<Expr>) -> Self {
        Self::Discrete { values }
    }
}

impl Default for Limits {
    fn default() -> Self {
        Self::Default
    }
}
