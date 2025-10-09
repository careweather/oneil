//! Model structures and collections for the Oneil programming language.

use std::collections::{HashMap, HashSet};

use crate::{
    model_import::{ReferenceMap, SubmodelImport, SubmodelMap, SubmodelName},
    parameter::{Parameter, ParameterCollection},
    reference::{Identifier, ModelPath, PythonPath},
    test::{Test, TestIndex},
};

/// Represents a single Oneil model containing parameters, tests, submodels, and imports.
#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    python_imports: HashSet<PythonPath>,
    submodels: SubmodelMap,
    references: ReferenceMap,
    parameters: ParameterCollection,
    tests: HashMap<TestIndex, Test>,
}

impl Model {
    /// Creates a new model with the specified components.
    #[must_use]
    pub const fn new(
        python_imports: HashSet<PythonPath>,
        submodels: SubmodelMap,
        references: ReferenceMap,
        parameters: ParameterCollection,
        tests: HashMap<TestIndex, Test>,
    ) -> Self {
        Self {
            python_imports,
            submodels,
            references,
            parameters,
            tests,
        }
    }

    /// Returns a reference to the set of Python imports for this model.
    #[must_use]
    pub const fn get_python_imports(&self) -> &HashSet<PythonPath> {
        &self.python_imports
    }

    /// Looks up a submodel by its identifier.
    #[must_use]
    pub fn get_submodel(&self, identifier: &SubmodelName) -> Option<&SubmodelImport> {
        self.submodels.get(identifier)
    }

    /// Returns a reference to all submodels in this model.
    #[must_use]
    pub const fn get_submodels(&self) -> &SubmodelMap {
        &self.submodels
    }

    /// Looks up a parameter by its identifier.
    #[must_use]
    pub fn get_parameter(&self, identifier: &Identifier) -> Option<&Parameter> {
        self.parameters.get(identifier)
    }

    /// Returns a reference to all parameters in this model.
    #[must_use]
    pub const fn get_parameters(&self) -> &ParameterCollection {
        &self.parameters
    }

    /// Returns a reference to all references in this model.
    #[must_use]
    pub const fn get_references(&self) -> &ReferenceMap {
        &self.references
    }

    /// Returns a reference to all tests in this model.
    #[must_use]
    pub const fn get_tests(&self) -> &HashMap<TestIndex, Test> {
        &self.tests
    }
}

/// A collection of models that can be managed together.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelCollection {
    initial_models: HashSet<ModelPath>,
    models: HashMap<ModelPath, Model>,
}

impl ModelCollection {
    /// Creates a new model collection with the specified initial models and model mapping.
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
    #[must_use]
    pub fn get_python_imports(&self) -> HashSet<&PythonPath> {
        self.models
            .values()
            .flat_map(|model| &model.python_imports)
            .collect()
    }

    /// Returns all models in the collection.
    #[must_use]
    pub const fn get_models(&self) -> &HashMap<ModelPath, Model> {
        &self.models
    }

    /// Returns the initial models (entry points).
    #[must_use]
    pub const fn get_initial_models(&self) -> &HashSet<ModelPath> {
        &self.initial_models
    }
}
