use std::collections::{HashMap, HashSet};

use crate::dependency::ParameterDependency;
use crate::dependency::{Dependency, ExternalImportList};
use crate::documentation::DocumentationMap;
use crate::path::ModulePath;
use crate::reference::Identifier;
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
    parameter_dependencies: HashMap<Identifier, HashSet<ParameterDependency>>,
}

impl Module {
    pub fn new(
        path: ModulePath,
        symbols: SymbolMap,
        tests: Tests,
        external_imports: ExternalImportList,
        documentation_map: DocumentationMap,
        dependencies: HashSet<Dependency>,
        parameter_dependencies: HashMap<Identifier, HashSet<ParameterDependency>>,
    ) -> Self {
        Self {
            path,
            symbols,
            tests,
            external_imports,
            documentation_map,
            dependencies,
            parameter_dependencies,
        }
    }

    pub fn path(&self) -> &ModulePath {
        &self.path
    }

    pub fn symbols(&self) -> &SymbolMap {
        &self.symbols
    }

    pub fn tests(&self) -> &Tests {
        &self.tests
    }

    pub fn external_imports(&self) -> &ExternalImportList {
        &self.external_imports
    }

    pub fn documentation_map(&self) -> &DocumentationMap {
        &self.documentation_map
    }

    pub fn dependencies(&self) -> &HashSet<Dependency> {
        &self.dependencies
    }

    pub fn parameter_dependencies(&self) -> &HashMap<Identifier, HashSet<ParameterDependency>> {
        &self.parameter_dependencies
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
        todo!("This may be replaced with a builder")
    }

    pub fn has_loaded_for(&self, module_path: &ModulePath) -> bool {
        self.modules.contains_key(module_path)
    }

    pub fn module(&self, module_path: &ModulePath) -> Option<&Module> {
        self.modules.get(module_path)
    }

    pub fn initial_modules(&self) -> &Vec<ModulePath> {
        &self.initial_modules
    }
}
