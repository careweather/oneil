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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Dependency {
    Python(PythonPath),
    Module(ModulePath),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SectionLabel {
    TopLevel,
    Subsection(String),
}

impl SectionLabel {
    pub fn new_top_level() -> Self {
        Self::TopLevel
    }

    pub fn new_subsection(label: String) -> Self {
        Self::Subsection(label)
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
pub enum SectionDecl {
    Test(TestIndex),
    Parameter(Identifier),
    InternalImport(Identifier),
    ExternalImport(PythonPath),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DocumentationMap {
    section_notes: HashMap<SectionLabel, ast::Note>,
    section_decls: HashMap<SectionLabel, Vec<SectionDecl>>,
}

impl DocumentationMap {
    pub fn new(
        section_notes: HashMap<SectionLabel, ast::Note>,
        section_decls: HashMap<SectionLabel, Vec<SectionDecl>>,
    ) -> Self {
        Self {
            section_notes,
            section_decls,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SectionData {
    note: Option<ast::Note>,
    items: Vec<SectionDecl>,
}

impl SectionData {
    pub fn new(note: Option<ast::Note>, items: Vec<SectionDecl>) -> Self {
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
pub struct Tests {
    model_tests: Vec<ast::Test>,
    dependency_tests: HashMap<ModulePath, TestInputs>,
}

impl Tests {
    pub fn new() -> Self {
        Self {
            model_tests: vec![],
            dependency_tests: HashMap::new(),
        }
    }

    pub fn add_model_test(&mut self, test: ast::Test) -> TestIndex {
        let test_index = self.model_tests.len();
        self.model_tests.push(test);
        TestIndex::new(test_index)
    }

    pub fn add_dependency_test(&mut self, module_path: ModulePath, inputs: TestInputs) {
        self.dependency_tests.insert(module_path, inputs);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestInputs(HashMap<Identifier, ast::Expr>);

impl TestInputs {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add_input(&mut self, ident: Identifier, expr: ast::Expr) {
        self.0.insert(ident, expr);
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
    dependencies: HashSet<Dependency>,
    dependent_modules: HashSet<ModulePath>,
}

impl Module {
    pub fn new(
        path: ModulePath,
        symbols: SymbolMap,
        tests: Tests,
        external_imports: ExternalImportMap,
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

    pub fn get_external_imports(&self) -> &ExternalImportMap {
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

// TODO: rename to ModuleGraph
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleCollection {
    initial_modules: Vec<ModulePath>,
    modules: HashMap<ModulePath, Module>,
}

impl ModuleCollection {
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
