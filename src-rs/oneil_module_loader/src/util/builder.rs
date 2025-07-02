use std::collections::{HashMap, HashSet};

use oneil_module::{
    module::{Module, ModuleCollection},
    parameter::{Parameter, ParameterCollection},
    reference::{Identifier, ModulePath},
};

use crate::error::collection::{ModuleErrorMap, ParameterErrorMap};

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleCollectionBuilder {
    initial_modules: HashSet<ModulePath>,
    modules: HashMap<ModulePath, Module>,
    visited_modules: HashSet<ModulePath>,
    errors: ModuleErrorMap,
}

impl ModuleCollectionBuilder {
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

    pub fn get_module(&self, module_path: &ModulePath) -> Option<&Module> {
        self.modules.get(module_path)
    }

    pub fn add_error(&mut self, module_path: ModulePath, error: ()) {
        self.errors.add_error(module_path, error);
    }

    pub fn add_module(&mut self, module_path: ModulePath, module: Module) {
        self.modules.insert(module_path, module);
    }
}

impl TryInto<ModuleCollection> for ModuleCollectionBuilder {
    type Error = (ModuleCollection, ());

    fn try_into(self) -> Result<ModuleCollection, (ModuleCollection, ())> {
        let module_collection = ModuleCollection::new(self.initial_modules, self.modules);
        if self.errors.is_empty() {
            Ok(module_collection)
        } else {
            Err((module_collection, ()))
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

    pub fn add_error(&mut self, identifier: Identifier, error: ()) {
        self.errors.add_error(identifier, error);
    }

    pub fn has_parameter(&self, identifier: &Identifier) -> bool {
        self.parameters.contains_key(identifier)
    }
}

impl TryInto<ParameterCollection> for ParameterCollectionBuilder {
    type Error = ();

    fn try_into(self) -> Result<ParameterCollection, ()> {
        if self.errors.is_empty() {
            Ok(ParameterCollection::new(self.parameters))
        } else {
            Err(())
        }
    }
}
