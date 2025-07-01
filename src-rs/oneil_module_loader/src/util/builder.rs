use std::collections::{HashMap, HashSet};

use oneil_module::{module::Module, reference::ModulePath};

use crate::error::collection::LoadErrorMap;

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleCollectionBuilder {
    initial_modules: Vec<ModulePath>,
    modules: HashMap<ModulePath, Module>,
    visited_modules: HashSet<ModulePath>,
    errors: LoadErrorMap,
}

impl ModuleCollectionBuilder {
    pub fn new(initial_modules: Vec<ModulePath>) -> Self {
        Self {
            initial_modules,
            modules: HashMap::new(),
            visited_modules: HashSet::new(),
            errors: LoadErrorMap::new(),
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
