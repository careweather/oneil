use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use oneil_ast as ast;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Reference {
    Identifier(Identifier),
    Accessor {
        parent: Identifier,
        component: Box<Reference>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier(String);

impl Identifier {
    pub fn new(ident: String) -> Self {
        Self(ident)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModulePath(PathBuf);

impl ModulePath {
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }

    pub fn join(&self, other: &str) -> PathBuf {
        self.0.join(other)
    }
}

impl AsRef<Path> for ModulePath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PythonPath(PathBuf);

impl PythonPath {
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }
}

impl AsRef<Path> for PythonPath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Dependency {
    Python(PythonPath),
    Module(ModulePath),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SectionLabel(String);

impl SectionLabel {
    pub fn new(label: String) -> Self {
        Self(label)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleReference {
    path: ModulePath,
    subcomponents: Vec<Identifier>,
}

impl ModuleReference {
    pub fn new(path: ModulePath, subcomponents: Vec<Identifier>) -> Self {
        Self {
            path,
            subcomponents,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestIndex(usize);

impl TestIndex {
    pub fn new(index: usize) -> Self {
        Self(index)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportIndex(usize);

impl ImportIndex {
    pub fn new(index: usize) -> Self {
        Self(index)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SectionItem {
    Test(TestIndex),
    Parameter(Identifier),
    InternalImport(Identifier),
    ExternalImport(ImportIndex),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DocumentationMap {
    top_section: SectionData,
    sub_sections: HashMap<SectionLabel, SectionData>,
}

impl DocumentationMap {
    pub fn new(top_section: SectionData, sub_sections: HashMap<SectionLabel, SectionData>) -> Self {
        Self {
            top_section,
            sub_sections,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SectionData {
    note: Option<ast::Note>,
    items: Vec<SectionItem>,
}

impl SectionData {
    pub fn new(note: Option<ast::Note>, items: Vec<SectionItem>) -> Self {
        Self { note, items }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    Parameter {
        dependencies: HashSet<Reference>,
        parameter: ast::Parameter,
    },
    Import(ModuleReference),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SymbolMap(HashMap<Identifier, Symbol>);

impl SymbolMap {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add_symbol(&mut self, ident: Identifier, symbol: Symbol) {
        self.0.insert(ident, symbol);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tests(Vec<ast::Test>);

impl Tests {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn add_test(&mut self, test: ast::Test) {
        self.0.push(test);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExternalImportMap(Vec<PythonPath>);

impl ExternalImportMap {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn add_import(&mut self, import_path: PythonPath) {
        self.0.push(import_path);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    path: ModulePath,
    symbols: SymbolMap,
    tests: Tests,
    external_imports: ExternalImportMap,
    documentation_map: DocumentationMap,
    dependencies: Vec<Dependency>,
}

impl Module {
    pub fn new(
        path: ModulePath,
        symbols: SymbolMap,
        tests: Tests,
        external_imports: ExternalImportMap,
        documentation_map: DocumentationMap,
        dependencies: Vec<Dependency>,
    ) -> Self {
        Self {
            path,
            symbols,
            tests,
            external_imports,
            documentation_map,
            dependencies,
        }
    }

    pub fn get_dependencies(&self) -> &[Dependency] {
        &self.dependencies
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

    pub fn get_external_imports(&self) -> &ExternalImportMap {
        &self.external_imports
    }

    pub fn get_documentation_map(&self) -> &DocumentationMap {
        &self.documentation_map
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleCollection {
    initial_module: Option<ModulePath>,
    modules: HashMap<ModulePath, Module>,
}

impl ModuleCollection {
    pub fn new() -> Self {
        Self {
            initial_module: None,
            modules: HashMap::new(),
        }
    }

    pub fn add_module(&mut self, module_path: &ModulePath, module: Module) {
        self.modules.insert(module_path.clone(), module);
    }

    pub fn has_loaded_for(&self, module_path: &ModulePath) -> bool {
        self.modules.contains_key(module_path)
    }
}
