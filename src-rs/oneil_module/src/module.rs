use std::collections::{HashMap, HashSet};

use crate::dependency::{Dependency, ExternalImportList};
use crate::documentation::DocumentationMap;
use crate::path::ModulePath;
use crate::symbol::SymbolMap;
use crate::test::Tests;

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    path: ModulePath,
    symbols: SymbolMap,
    tests: Tests,
    external_imports: ExternalImportList,
    documentation_map: DocumentationMap,
    dependencies: HashSet<Dependency>,
    dependent_modules: HashSet<ModulePath>,
}

impl Module {
    pub fn new(
        path: ModulePath,
        symbols: SymbolMap,
        tests: Tests,
        external_imports: ExternalImportList,
        documentation_map: DocumentationMap,
        dependencies: HashSet<Dependency>,
    ) -> Self {
        Self {
            path,
            symbols,
            tests,
            external_imports,
            documentation_map,
            dependencies,
            dependent_modules: HashSet::new(),
        }
    }

    pub fn get_path(&self) -> &ModulePath {
        &self.path
    }

    pub fn get_symbols(&self) -> &SymbolMap {
        &self.symbols
    }

    pub fn get_tests(&self) -> &Tests {
        &self.tests
    }

    pub fn get_external_imports(&self) -> &ExternalImportList {
        &self.external_imports
    }

    pub fn get_documentation_map(&self) -> &DocumentationMap {
        &self.documentation_map
    }

    pub fn get_dependencies(&self) -> &HashSet<Dependency> {
        &self.dependencies
    }

    pub fn get_dependent_modules(&self) -> &HashSet<ModulePath> {
        &self.dependent_modules
    }

    pub fn add_dependent_module(&mut self, module_path: ModulePath) {
        self.dependent_modules.insert(module_path);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleGraph {
    initial_modules: Vec<ModulePath>,
    modules: HashMap<ModulePath, Module>,
}

impl ModuleGraph {
    pub fn new(initial_modules: Vec<ModulePath>) -> Self {
        Self {
            initial_modules,
            modules: HashMap::new(),
        }
    }

    pub fn add_module(&mut self, module_path: &ModulePath, module: Module) {
        self.modules.insert(module_path.clone(), module);
    }

    pub fn add_dependent_module(
        &mut self,
        module_path: &ModulePath,
        dependent_module_path: ModulePath,
    ) {
        self.modules
            .get_mut(module_path)
            .unwrap()
            .add_dependent_module(dependent_module_path);
    }

    pub fn has_loaded_for(&self, module_path: &ModulePath) -> bool {
        self.modules.contains_key(module_path)
    }
}
