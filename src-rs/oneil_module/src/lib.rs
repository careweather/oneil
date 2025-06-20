use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use oneil_ast as ast;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident(String);

impl Ident {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PythonPath(PathBuf);

impl PythonPath {
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SectionLabel(String);

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleReference {
    path: ModulePath,
    subcomponents: Vec<Ident>,
}

impl ModuleReference {
    pub fn new(path: ModulePath, subcomponents: Vec<Ident>) -> Self {
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
    Parameter(Ident),
    InternalImport(Ident),
    ExternalImport(ImportIndex),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DocumentationMap {
    top_section: SectionData,
    sub_sections: HashMap<SectionLabel, SectionData>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SectionData {
    note: Option<ast::Note>,
    items: Vec<SectionItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    Parameter(ast::Parameter),
    Import(ModuleReference),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    symbols: SymbolMap,
    tests: Tests,
    external_imports: ExternalImportMap,
    documentation_map: DocumentationMap,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SymbolMap(HashMap<Ident, Symbol>);

impl SymbolMap {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add_symbol(&mut self, ident: Ident, symbol: Symbol) {
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

impl Module {
    pub fn new(
        symbols: SymbolMap,
        tests: Tests,
        external_imports: ExternalImportMap,
        documentation_map: DocumentationMap,
    ) -> Self {
        Self {
            symbols,
            tests,
            external_imports,
            documentation_map,
        }
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
