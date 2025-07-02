use std::collections::{HashMap, HashSet};

use oneil_module::{
    module::{Module, ModuleCollection},
    parameter::{Parameter, ParameterCollection},
    reference::{Identifier, ModulePath},
};

use crate::error::{
    LoadError, ResolutionError, ResolutionErrorSource,
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

    pub fn add_error(&mut self, module_path: ModulePath, error: LoadError<Ps, Py>) {
        self.errors.add_error(module_path, error);
    }

    pub fn add_error_list<I, J>(&mut self, module_path: ModulePath, errors: I)
    where
        I: IntoIterator<Item = J>,
        J: Into<LoadError<Ps, Py>>,
    {
        for error in errors {
            self.add_error(module_path.clone(), error.into());
        }
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

    pub fn add_error(&mut self, identifier: Identifier, error: ResolutionErrorSource) {
        self.errors.add_error(identifier, error);
    }

    pub fn has_parameter(&self, identifier: &Identifier) -> bool {
        self.parameters.contains_key(identifier)
    }
}

impl TryInto<ParameterCollection> for ParameterCollectionBuilder {
    type Error = (ParameterCollection, Vec<ResolutionError>);

    fn try_into(self) -> Result<ParameterCollection, (ParameterCollection, Vec<ResolutionError>)> {
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
