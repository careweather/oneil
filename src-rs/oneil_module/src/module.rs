use std::collections::{HashMap, HashSet};

use crate::{
    parameter::{Parameter, ParameterCollection},
    reference::{Identifier, ModulePath, PythonPath},
    test::{ModelTest, SubmodelTest},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    python_imports: HashSet<PythonPath>,
    submodels: HashMap<Identifier, ModulePath>,
    parameters: ParameterCollection,
    model_tests: Vec<ModelTest>,
    submodel_tests: Vec<SubmodelTest>,
}

impl Module {
    pub fn new(
        python_imports: HashSet<PythonPath>,
        submodels: HashMap<Identifier, ModulePath>,
        parameters: ParameterCollection,
        model_tests: Vec<ModelTest>,
        submodel_tests: Vec<SubmodelTest>,
    ) -> Self {
        Self {
            python_imports,
            submodels,
            parameters,
            model_tests,
            submodel_tests,
        }
    }

    pub fn get_python_imports(&self) -> &HashSet<PythonPath> {
        &self.python_imports
    }

    pub fn get_submodel(&self, identifier: &Identifier) -> Option<&ModulePath> {
        self.submodels.get(identifier)
    }

    pub fn get_parameter(&self, identifier: &Identifier) -> Option<&Parameter> {
        self.parameters.get(identifier)
    }

    pub fn get_model_tests(&self) -> &Vec<ModelTest> {
        &self.model_tests
    }

    pub fn get_submodel_tests(&self) -> &Vec<SubmodelTest> {
        &self.submodel_tests
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleCollection {
    initial_modules: HashSet<ModulePath>,
    modules: HashMap<ModulePath, Module>,
}

impl ModuleCollection {
    pub fn new(initial_modules: HashSet<ModulePath>, modules: HashMap<ModulePath, Module>) -> Self {
        Self {
            initial_modules,
            modules,
        }
    }

    /// Returns all python imports from modules in the collection
    pub fn get_python_imports(&self) -> HashSet<&PythonPath> {
        self.modules
            .values()
            .flat_map(|module| module.python_imports.iter())
            .collect()
    }
}
