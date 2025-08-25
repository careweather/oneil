//! Model structures and collections for the Oneil programming language.
//!
//! This module defines the core data structures for representing Oneil models
//! as an intermediate representation (IR), including their parameters, tests, submodels,
//! and Python imports. Models are the primary organizational unit in Oneil,
//! containing all the components needed to define a model or submodel.

use std::collections::{HashMap, HashSet};

use crate::{
    parameter::{Parameter, ParameterCollection},
    reference::{Identifier, ModelPath, PythonPath},
    span::Span,
    test::{Test, TestIndex},
};

/// Represents a single Oneil model containing parameters, tests, submodels, and imports.
///
/// A model is the fundamental building block in Oneil, representing either a complete
/// model or a reusable submodel. Each model can contain:
///
/// - **Parameters**: Named values with expressions and constraints
/// - **Tests**: Validation rules for the model's behavior
/// - **Submodels**: References to other models that this model depends on
/// - **Python Imports**: External Python modules that provide additional functionality
///
/// Models are immutable by design, following functional programming principles.
///
/// Note that the `Span` for python imports and submodels is the span of the
/// identifier.
#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    python_imports: HashMap<PythonPath, Span>,
    submodels: HashMap<Identifier, (ModelPath, Span)>,
    parameters: ParameterCollection,
    tests: HashMap<TestIndex, Test>,
}

impl Model {
    /// Creates a new model with the specified components.
    ///
    /// # Arguments
    ///
    /// * `python_imports` - Mapping of Python modules to import to their identifier spans
    /// * `submodels` - Mapping of submodel identifiers to their model paths and identifier spans
    /// * `parameters` - Collection of parameters defined in this model
    /// * `tests` - Tests for the entire model
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{model::Model, parameter::ParameterCollection, span::WithSpan, reference::{Identifier, ModelPath, PythonPath}};
    /// use std::collections::{HashMap, HashSet};
    ///
    /// let model = Model::new(
    ///     HashMap::new(), // no Python imports
    ///     HashMap::new(),  // no submodels
    ///     ParameterCollection::new(HashMap::new()),
    ///     HashMap::new(),  // no tests
    /// );
    /// ```
    #[must_use]
    pub const fn new(
        python_imports: HashMap<PythonPath, Span>,
        submodels: HashMap<Identifier, (ModelPath, Span)>,
        parameters: ParameterCollection,
        tests: HashMap<TestIndex, Test>,
    ) -> Self {
        Self {
            python_imports,
            submodels,
            parameters,
            tests,
        }
    }

    /// Returns a reference to the set of Python imports for this model.
    ///
    /// Python imports allow models to use external Python functionality
    /// for complex calculations or data processing.
    #[must_use]
    pub const fn get_python_imports(&self) -> &HashMap<PythonPath, Span> {
        &self.python_imports
    }

    /// Looks up a submodel by its identifier.
    ///
    /// Returns `Some(ModelPath)` if the submodel exists, `None` otherwise.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The identifier of the submodel to look up
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{model::Model, parameter::ParameterCollection, span::WithSpan, reference::{Identifier, ModelPath, PythonPath}};
    /// use std::collections::{HashMap, HashSet};
    ///
    /// let mut submodels = HashMap::new();
    /// submodels.insert(Identifier::new("sub"), (ModelPath::new("submodel"), oneil_ir::span::Span::new(0, 0)));
    ///
    /// let model = Model::new(
    ///     HashMap::new(),
    ///     submodels,
    ///     ParameterCollection::new(HashMap::new()),
    ///     HashMap::new(),
    /// );
    ///
    /// assert!(model.get_submodel(&Identifier::new("sub")).is_some());
    /// assert!(model.get_submodel(&Identifier::new("nonexistent")).is_none());
    /// ```
    #[must_use]
    pub fn get_submodel(&self, identifier: &Identifier) -> Option<&(ModelPath, Span)> {
        self.submodels.get(identifier)
    }

    /// Returns a reference to all submodels in this model.
    ///
    /// # Returns
    ///
    /// A reference to the mapping of submodel identifiers to their corresponding model paths.
    #[must_use]
    pub const fn get_submodels(&self) -> &HashMap<Identifier, (ModelPath, Span)> {
        &self.submodels
    }

    /// Looks up a parameter by its identifier.
    ///
    /// Returns `Some(Parameter)` if the parameter exists, `None` otherwise.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The identifier of the parameter to look up
    #[must_use]
    pub fn get_parameter(&self, identifier: &Identifier) -> Option<&Parameter> {
        self.parameters.get(identifier)
    }

    /// Returns a reference to all parameters in this model.
    ///
    /// # Returns
    ///
    /// A reference to the parameter collection.
    #[must_use]
    pub const fn get_parameters(&self) -> &ParameterCollection {
        &self.parameters
    }

    /// Returns a reference to all tests in this model.
    ///
    /// Tests validate the behavior of the entire model and are
    /// indexed by test indices for easy lookup.
    #[must_use]
    pub const fn get_tests(&self) -> &HashMap<TestIndex, Test> {
        &self.tests
    }

    /// Checks if this model is empty (contains no components).
    ///
    /// A model is considered empty if it has no Python imports, submodels,
    /// parameters, or tests.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{model::Model, parameter::ParameterCollection};
    /// use std::collections::{HashMap, HashSet};
    ///
    /// let empty_model = Model::new(
    ///     HashMap::new(),
    ///     HashMap::new(),
    ///     ParameterCollection::new(HashMap::new()),
    ///     HashMap::new(),
    /// );
    ///
    /// assert!(empty_model.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.python_imports.is_empty()
            && self.submodels.is_empty()
            && self.parameters.is_empty()
            && self.tests.is_empty()
    }
}

/// A collection of models that can be managed together.
///
/// `ModelCollection` provides a way to organize and manage multiple models,
/// particularly useful for handling model dependencies and resolving imports.
/// It maintains a set of initial models (entry points) and a mapping of all
/// available models.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelCollection {
    initial_models: HashSet<ModelPath>,
    models: HashMap<ModelPath, Model>,
}

impl ModelCollection {
    /// Creates a new model collection with the specified initial models and model mapping.
    ///
    /// # Arguments
    ///
    /// * `initial_models` - Set of model paths that serve as entry points
    /// * `models` - Mapping of model paths to their corresponding models
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{model::{ModelCollection, Model}, reference::ModelPath, parameter::ParameterCollection};
    /// use std::collections::{HashMap, HashSet};
    ///
    /// let mut initial_models = HashSet::new();
    /// initial_models.insert(ModelPath::new("main"));
    ///
    /// let mut models = HashMap::new();
    /// models.insert(ModelPath::new("main"), Model::new(
    ///     HashMap::new(),
    ///     HashMap::new(),
    ///     ParameterCollection::new(HashMap::new()),
    ///     HashMap::new(),
    /// ));
    ///
    /// let collection = ModelCollection::new(initial_models, models);
    /// ```
    #[must_use]
    pub const fn new(
        initial_models: HashSet<ModelPath>,
        models: HashMap<ModelPath, Model>,
    ) -> Self {
        Self {
            initial_models,
            models,
        }
    }

    /// Returns all Python imports from all models in the collection.
    ///
    /// This method aggregates Python imports from all models, which is useful
    /// for dependency analysis and ensuring all required Python modules are available.
    ///
    /// # Returns
    ///
    /// A set of references to all Python paths imported across all models.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{model::{ModelCollection, Model}, reference::{ModelPath, PythonPath}, parameter::ParameterCollection, span::WithSpan};
    /// use std::collections::{HashMap, HashSet};
    /// use std::path::PathBuf;
    ///
    /// let mut initial_models = HashSet::new();
    /// initial_models.insert(ModelPath::new("main"));
    ///
    /// let mut python_imports = HashMap::new();
    /// python_imports.insert(PythonPath::new(PathBuf::from("math")), oneil_ir::span::Span::new(0, 0));
    ///
    /// let mut models = HashMap::new();
    /// models.insert(ModelPath::new("main"), Model::new(
    ///     python_imports,
    ///     HashMap::new(),
    ///     ParameterCollection::new(HashMap::new()),
    ///     HashMap::new(),
    /// ));
    ///
    /// let collection = ModelCollection::new(initial_models, models);
    /// let imports = collection.get_python_imports();
    /// assert_eq!(imports.len(), 1);
    /// ```
    #[must_use]
    pub fn get_python_imports(&self) -> HashSet<&PythonPath> {
        self.models
            .values()
            .flat_map(|model| model.python_imports.keys())
            .collect()
    }

    /// Returns all models in the collection.
    ///
    /// This method provides access to all models in the collection.
    ///
    /// # Returns
    ///
    /// A reference to the mapping of model paths to their corresponding models.
    #[must_use]
    pub const fn get_models(&self) -> &HashMap<ModelPath, Model> {
        &self.models
    }

    /// Returns the initial models (entry points).
    ///
    /// Initial models are the entry points for the model collection,
    /// typically representing the main models that were originally loaded.
    ///
    /// # Returns
    ///
    /// A reference to the set of initial model paths.
    #[must_use]
    pub const fn get_initial_models(&self) -> &HashSet<ModelPath> {
        &self.initial_models
    }
}
