//! Output types for the Oneil programming language.
//!
//! These types are used to represent the results of
//! evaluating Oneil models.

pub mod dependency;
pub mod tree;

use std::path::PathBuf;

use indexmap::IndexMap;

use oneil_shared::span::Span;

use crate::{output::dependency::DependencySet, value::Value};

/// The result of evaluating a model.
///
/// This structure represents a fully evaluated model, containing all evaluated
/// parameters, tests, and recursively evaluated submodels. It is produced by
/// the evaluation process and can be used for output, further processing, or
/// analysis.
#[derive(Debug, Clone)]
pub struct Model {
    /// The file path of the model that was evaluated.
    pub path: PathBuf,
    /// A map of submodel names to their reference names.
    pub submodels: IndexMap<String, String>,
    /// A map of reference names to their evaluated results.
    ///
    /// References are evaluated recursively, so each entry contains a fully
    /// evaluated `Model` structure.
    pub references: IndexMap<String, PathBuf>,
    /// A map of parameter identifiers to their evaluated results.
    ///
    /// Parameters are stored by their identifier (name) and contain their
    /// evaluated values, units, and metadata.
    pub parameters: IndexMap<String, Parameter>,
    /// A list of evaluated test results.
    ///
    /// Tests are evaluated expressions that verify model behavior. Each test
    /// contains the evaluated value and the span of the original expression.
    pub tests: Vec<Test>,
}

/// The result of evaluating a test expression.
///
/// Tests are boolean expressions that verify expected behavior in a model.
/// This structure contains the evaluated value (which should be a boolean)
/// and the source location of the test expression.
#[derive(Debug, Clone)]
pub struct Test {
    /// The source span of the test expression.
    pub expr_span: Span,
    /// The evaluated value of the test expression.
    pub result: TestResult,
}

impl Test {
    /// Returns whether the test passed.
    #[must_use]
    pub const fn passed(&self) -> bool {
        matches!(self.result, TestResult::Passed)
    }
}

/// The result of evaluating a test.
#[derive(Debug, Clone)]
pub enum TestResult {
    /// The test passed.
    Passed,
    /// The test failed.
    Failed {
        /// The values of the test dependencies.
        debug_info: Box<DebugInfo>,
    },
}

/// The result of evaluating a parameter.
///
/// Parameters are the primary data elements in a model. This structure
/// contains the evaluated value, associated unit (if any), and metadata about
/// the parameter such as whether it's a performance parameter and its
/// dependencies.
#[derive(Debug, Clone)]
pub struct Parameter {
    /// The identifier (name) of the parameter.
    pub ident: String,
    /// The human-readable label for the parameter.
    pub label: String,
    /// The evaluated value of the parameter.
    pub value: Value,
    /// The print level for this parameter.
    ///
    /// This determines the level of debugging/tracing information that should
    /// be generated for this parameter during output.
    pub print_level: PrintLevel,
    /// The debug information for this parameter, if requested.
    pub debug_info: Option<DebugInfo>,
    /// The dependencies of this parameter.
    pub dependencies: DependencySet,
    /// The span of the parameter expression.
    pub expr_span: Span,
}

impl Parameter {
    /// Returns whether this parameter should be printed at
    /// the given print level.
    #[must_use]
    pub fn should_print(&self, print_level: PrintLevel) -> bool {
        self.print_level >= print_level
    }
}

/// Debug information for a parameter.
#[derive(Debug, Clone)]
pub struct DebugInfo {
    /// The values of the builtin dependencies at the time the parameter was evaluated.
    pub builtin_dependency_values: IndexMap<String, Value>,
    /// The values of the parameter dependencies at the time the parameter was evaluated.
    pub parameter_dependency_values: IndexMap<String, Value>,
    /// The values of the external dependencies at the time the parameter was evaluated.
    pub external_dependency_values: IndexMap<(String, String), Value>,
}

/// The trace level for debugging and diagnostic output.
///
/// Trace levels control the verbosity of debugging information during model
/// evaluation. Higher levels provide more detailed information.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PrintLevel {
    /// No output.
    None,
    /// Basic tracing output.
    Trace,
    /// Performance output.
    Performance,
}
