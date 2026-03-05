//! Model structures and collections for the Oneil programming language.

use indexmap::IndexMap;

use crate::{
    ModelPath,
    model_import::{ReferenceImport, ReferenceName, SubmodelImport, SubmodelName},
    parameter::{Parameter, ParameterName},
    python_import::PythonImport,
    reference::PythonPath,
    test::{Test, TestIndex},
};

/// Represents a single Oneil model containing parameters, tests, submodels, and imports.
#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    path: ModelPath,
    python_imports: IndexMap<PythonPath, PythonImport>,
    submodels: IndexMap<SubmodelName, SubmodelImport>,
    references: IndexMap<ReferenceName, ReferenceImport>,
    parameters: IndexMap<ParameterName, Parameter>,
    tests: IndexMap<TestIndex, Test>,
}

impl Model {
    /// Creates a new model with the specified components.
    #[must_use]
    pub const fn new(
        path: ModelPath,
        python_imports: IndexMap<PythonPath, PythonImport>,
        submodels: IndexMap<SubmodelName, SubmodelImport>,
        references: IndexMap<ReferenceName, ReferenceImport>,
        parameters: IndexMap<ParameterName, Parameter>,
        tests: IndexMap<TestIndex, Test>,
    ) -> Self {
        Self {
            path,
            python_imports,
            submodels,
            references,
            parameters,
            tests,
        }
    }

    /// Returns the path of this model.
    #[must_use]
    pub const fn get_path(&self) -> &ModelPath {
        &self.path
    }

    /// Returns a reference to the set of Python imports for this model.
    #[must_use]
    pub const fn get_python_imports(&self) -> &IndexMap<PythonPath, PythonImport> {
        &self.python_imports
    }

    /// Looks up a submodel by its identifier.
    #[must_use]
    pub fn get_submodel(&self, identifier: &SubmodelName) -> Option<&SubmodelImport> {
        self.submodels.get(identifier)
    }

    /// Returns the reference that a submodel is associated with.
    #[must_use]
    #[expect(
        clippy::missing_panics_doc,
        reason = "the panic is only caused by breaking an internal invariant"
    )]
    pub fn get_submodel_reference(&self, identifier: &SubmodelName) -> Option<&ReferenceImport> {
        let submodel = self.submodels.get(identifier)?;
        let reference = self
            .references
            .get(submodel.reference_name())
            .expect("reference corresponding to submodel should exist");
        Some(reference)
    }

    /// Returns a reference to all submodels in this model.
    #[must_use]
    pub const fn get_submodels(&self) -> &IndexMap<SubmodelName, SubmodelImport> {
        &self.submodels
    }

    /// Looks up a parameter by its identifier.
    #[must_use]
    pub fn get_parameter(&self, identifier: &ParameterName) -> Option<&Parameter> {
        self.parameters.get(identifier)
    }

    /// Returns a reference to all parameters in this model.
    #[must_use]
    pub const fn get_parameters(&self) -> &IndexMap<ParameterName, Parameter> {
        &self.parameters
    }

    /// Looks up a reference by its identifier.
    #[must_use]
    pub fn get_reference(&self, identifier: &ReferenceName) -> Option<&ReferenceImport> {
        self.references.get(identifier)
    }

    /// Returns a reference to all references in this model.
    #[must_use]
    pub const fn get_references(&self) -> &IndexMap<ReferenceName, ReferenceImport> {
        &self.references
    }

    /// Returns a reference to all tests in this model.
    #[must_use]
    pub const fn get_tests(&self) -> &IndexMap<TestIndex, Test> {
        &self.tests
    }

    /// Adds a Python import to this model.
    pub fn add_python_import(&mut self, path: PythonPath, import: PythonImport) {
        self.python_imports.insert(path, import);
    }

    /// Adds a reference to this model.
    pub fn add_reference(&mut self, name: ReferenceName, import: ReferenceImport) {
        self.references.insert(name, import);
    }

    /// Adds a submodel to this model.
    pub fn add_submodel(&mut self, name: SubmodelName, import: SubmodelImport) {
        self.submodels.insert(name, import);
    }

    /// Adds a parameter to this model.
    pub fn add_parameter(&mut self, name: ParameterName, parameter: Parameter) {
        self.parameters.insert(name, parameter);
    }

    /// Adds a test to this model.
    pub fn add_test(&mut self, index: TestIndex, test: Test) {
        self.tests.insert(index, test);
    }
}
