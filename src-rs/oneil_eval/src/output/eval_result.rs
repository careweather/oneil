//! Result types for model evaluation.
//!
//! This module contains the data structures that represent the results of
//! evaluating Oneil models, including evaluated parameters, tests, and
//! hierarchical model structures.

use std::path::{Path, PathBuf};

use indexmap::IndexMap;

use oneil_shared::span::Span;

use crate::{ModelError, output::dependency::DependencySet, value::Value};

/// The result of evaluating a model hierarchy.
///
/// This structure holds the top-level model path and
/// a map of model paths to their evaluation results (model data and errors).
///
/// In order to get the top-level model data, use the `get_top_model` method.
#[derive(Debug, Clone)]
pub struct EvalResult {
    top_model_path: PathBuf,
    model_info: IndexMap<PathBuf, (Model, Vec<ModelError>)>,
}

impl EvalResult {
    /// Creates a new `EvalResult` for the given top-level model path.
    #[must_use]
    pub(crate) fn new(top_model_path: PathBuf) -> Self {
        Self {
            top_model_path,
            model_info: IndexMap::new(),
        }
    }

    /// Adds a model's evaluation result to this collection.
    pub(crate) fn add_model(&mut self, model_path: PathBuf, model: Model, errors: Vec<ModelError>) {
        self.model_info.insert(model_path, (model, errors));
    }

    /// Checks whether a model at the given path has been visited and evaluated.
    #[must_use]
    pub(crate) fn model_is_visited(&self, model_path: impl AsRef<Path>) -> bool {
        self.model_info.contains_key(model_path.as_ref())
    }

    /// Returns a reference to the top-level model.
    ///
    /// # Panics
    ///
    /// Panics if the top model has not been visited
    /// and added to this result. This should never be
    /// the case as long as creating the `EvalResult`
    /// resolves successfully.
    #[must_use]
    pub fn get_top_model(&self) -> ModelReference<'_> {
        let top_model = &self
            .model_info
            .get(&self.top_model_path)
            .expect("top model should be visited");

        ModelReference {
            model: &top_model.0,
            model_collection: &self.model_info,
        }
    }

    /// Returns all errors from all models in this evaluation result.
    #[must_use]
    pub fn get_errors(&self) -> Vec<&ModelError> {
        self.model_info
            .values()
            .flat_map(|(_, errors)| errors.iter())
            .collect()
    }
}

/// A reference to an evaluated model within a model hierarchy.
///
/// This stores a reference to a model and a reference to the
/// entire model collection.
#[derive(Debug, Clone, Copy)]
pub struct ModelReference<'result> {
    model: &'result Model,
    model_collection: &'result IndexMap<PathBuf, (Model, Vec<ModelError>)>,
}

impl<'result> ModelReference<'result> {
    /// Returns the file path of this model.
    #[must_use]
    pub fn path(&self) -> &'result Path {
        self.model.path.as_path()
    }

    /// Returns a map of submodel names to their model references.
    ///
    /// # Panics
    ///
    /// Panics if any submodel has not been visited and
    /// added to the model collection. This should never be
    /// the case as long as creating the `EvalResult`
    /// resolves successfully.
    #[must_use]
    pub fn submodels(&self) -> IndexMap<&'result str, Self> {
        self.model
            .submodels
            .iter()
            .map(|(name, path)| {
                let (model, _) = self
                    .model_collection
                    .get(path)
                    .expect("submodel should be visited");

                (
                    name.as_str(),
                    Self {
                        model,
                        model_collection: self.model_collection,
                    },
                )
            })
            .collect()
    }

    /// Returns a map of reference names to their model references.
    ///
    /// # Panics
    ///
    /// Panics if any reference has not been visited and
    /// added to the model collection. This should never be
    /// the case as long as creating the `EvalResult`
    /// resolves successfully.
    #[must_use]
    pub fn references(&self) -> IndexMap<&'result str, Self> {
        self.model
            .references
            .iter()
            .map(|(name, path)| {
                let (model, _) = self
                    .model_collection
                    .get(path)
                    .expect("reference should be visited");

                (
                    name.as_str(),
                    Self {
                        model,
                        model_collection: self.model_collection,
                    },
                )
            })
            .collect()
    }

    /// Returns a map of parameter names to their evaluated parameter data.
    #[must_use]
    pub fn parameters(&self) -> IndexMap<&'result str, &'result Parameter> {
        self.model
            .parameters
            .iter()
            .map(|(name, parameter)| (name.as_str(), parameter))
            .collect()
    }

    /// Returns the list of evaluated test results for this model.
    #[must_use]
    pub fn tests(&self) -> Vec<&'result Test> {
        self.model.tests.iter().collect()
    }
}

/// The result of evaluating a model.
///
/// This structure represents a fully evaluated model, containing all evaluated
/// parameters, tests, and recursively evaluated submodels. It is produced by
/// the evaluation process and can be used for output, further processing, or
/// analysis.
#[derive(Debug, Clone)]
pub(crate) struct Model {
    /// The file path of the model that was evaluated.
    pub path: PathBuf,
    /// A map of submodel names to their evaluated results.
    ///
    /// Submodels are evaluated recursively, so each entry contains a fully
    /// evaluated `Model` structure.
    pub submodels: IndexMap<String, PathBuf>,
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
