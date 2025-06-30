use std::collections::{HashMap, HashSet};

use crate::{
    dependency::{Dependency, ExternalImportList, ParameterDependency, TestDependency},
    documentation::DocumentationMap,
    path::ModulePath,
    reference::Identifier,
    symbol::SymbolMap,
    test::{TestIndex, Tests},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    path: ModulePath,
    symbols: SymbolMap,
    tests: Tests,
    external_imports: ExternalImportList,
    documentation_map: DocumentationMap,
    dependencies: HashSet<Dependency>,
    parameter_dependencies: HashMap<Identifier, HashSet<ParameterDependency>>,
    test_dependencies: HashMap<TestIndex, HashSet<TestDependency>>,
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
        test_dependencies: HashMap<TestIndex, HashSet<TestDependency>>,
    ) -> Self {
        Self {
            path,
            symbols,
            tests,
            external_imports,
            documentation_map,
            dependencies,
            parameter_dependencies,
            test_dependencies,
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

    pub fn test_dependencies(&self) -> &HashMap<TestIndex, HashSet<TestDependency>> {
        &self.test_dependencies
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleCollection {
    initial_modules: Vec<ModulePath>,
    modules: HashMap<ModulePath, Module>,
}

impl ModuleCollection {
    pub fn new(initial_modules: Vec<ModulePath>, modules: HashMap<ModulePath, Module>) -> Self {
        Self {
            initial_modules,
            modules,
        }
    }

    pub fn module(&self, module_path: &ModulePath) -> Option<&Module> {
        self.modules.get(module_path)
    }

    pub fn initial_modules(&self) -> &Vec<ModulePath> {
        &self.initial_modules
    }
}
