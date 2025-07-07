use std::collections::{HashMap, HashSet};

use oneil_module::{
    module::{Module, ModuleCollection},
    parameter::{Parameter, ParameterCollection},
    reference::{Identifier, ModulePath, PythonPath},
};

use crate::error::{
    LoadError, ParameterResolutionError,
    collection::{ModuleErrorMap, ParameterErrorMap},
};

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleCollectionBuilder<Ps, Py> {
    initial_modules: HashSet<ModulePath>,
    modules: HashMap<ModulePath, Module>,
    visited_modules: HashSet<ModulePath>,
    errors: ModuleErrorMap<Ps, Py>,
}

impl<Ps, Py> ModuleCollectionBuilder<Ps, Py> {
    pub fn new(initial_modules: HashSet<ModulePath>) -> Self {
        Self {
            initial_modules,
            modules: HashMap::new(),
            visited_modules: HashSet::new(),
            errors: ModuleErrorMap::new(),
        }
    }

    pub fn module_has_been_visited(&self, module_path: &ModulePath) -> bool {
        self.visited_modules.contains(module_path)
    }

    pub fn mark_module_as_visited(&mut self, module_path: &ModulePath) {
        self.visited_modules.insert(module_path.clone());
    }

    pub fn get_modules(&self) -> &HashMap<ModulePath, Module> {
        &self.modules
    }

    pub fn get_modules_with_errors(&self) -> HashSet<&ModulePath> {
        self.errors.get_modules_with_errors()
    }

    pub fn add_module_error(&mut self, module_path: ModulePath, error: LoadError<Ps>) {
        self.errors.add_error(module_path, error);
    }

    pub fn add_python_error(&mut self, python_path: PythonPath, error: Py) {
        self.errors.add_import_error(python_path, error);
    }

    pub fn add_module(&mut self, module_path: ModulePath, module: Module) {
        self.modules.insert(module_path, module);
    }
}

impl<Ps, Py> TryInto<ModuleCollection> for ModuleCollectionBuilder<Ps, Py> {
    type Error = (ModuleCollection, ModuleErrorMap<Ps, Py>);

    fn try_into(self) -> Result<ModuleCollection, (ModuleCollection, ModuleErrorMap<Ps, Py>)> {
        let module_collection = ModuleCollection::new(self.initial_modules, self.modules);
        if self.errors.is_empty() {
            Ok(module_collection)
        } else {
            Err((module_collection, self.errors))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParameterCollectionBuilder {
    parameters: HashMap<Identifier, Parameter>,
    errors: ParameterErrorMap,
}

impl ParameterCollectionBuilder {
    pub fn new() -> Self {
        Self {
            parameters: HashMap::new(),
            errors: ParameterErrorMap::new(),
        }
    }

    pub fn add_parameter(&mut self, identifier: Identifier, parameter: Parameter) {
        self.parameters.insert(identifier, parameter);
    }

    pub fn add_error(&mut self, identifier: Identifier, error: ParameterResolutionError) {
        self.errors.add_error(identifier, error);
    }

    pub fn add_error_list<I>(&mut self, identifier: Identifier, errors: I)
    where
        I: IntoIterator<Item = ParameterResolutionError>,
    {
        for error in errors {
            self.add_error(identifier.clone(), error);
        }
    }

    pub fn get_defined_parameters(&self) -> &HashMap<Identifier, Parameter> {
        &self.parameters
    }

    pub fn get_parameters_with_errors(&self) -> HashSet<&Identifier> {
        self.errors.get_parameters_with_errors()
    }
}

impl TryInto<ParameterCollection> for ParameterCollectionBuilder {
    type Error = (
        ParameterCollection,
        HashMap<Identifier, Vec<ParameterResolutionError>>,
    );

    fn try_into(
        self,
    ) -> Result<
        ParameterCollection,
        (
            ParameterCollection,
            HashMap<Identifier, Vec<ParameterResolutionError>>,
        ),
    > {
        if self.errors.is_empty() {
            Ok(ParameterCollection::new(self.parameters))
        } else {
            Err((
                ParameterCollection::new(self.parameters),
                self.errors.into(),
            ))
        }
    }
}
