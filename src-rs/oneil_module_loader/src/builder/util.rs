use std::collections::{HashMap, HashSet};

use oneil_ast as ast;
use oneil_module::{
    Dependency, DocumentationMap, ExternalImportList, Identifier, Module, ModulePath,
    ParameterDependency, PythonPath, SectionDecl, SectionLabel, Symbol, SymbolMap, TestIndex,
    TestInputs, Tests,
};

/// A builder pattern implementation for constructing Module instances.
///
/// This struct provides a mutable API for incrementally building up a Module's components
/// (symbols, tests, imports, etc). Once construction is complete, the builder can be
/// converted into an immutable Module via `into_module()`.
///
/// This approach allows for a more ergonomic module construction process compared to
/// working directly with the immutable Module type.
pub struct ModuleBuilder {
    module_path: ModulePath,
    symbols: HashMap<Identifier, Symbol>,
    model_tests: Vec<ast::Test>,
    dependency_tests: HashMap<ModulePath, TestInputs>,
    external_imports: Vec<PythonPath>,
    section_notes: HashMap<SectionLabel, ast::Note>,
    section_items: HashMap<SectionLabel, Vec<SectionDecl>>,
    dependencies: HashSet<Dependency>,
    parameter_dependencies: HashMap<Identifier, HashSet<ParameterDependency>>,
}

impl ModuleBuilder {
    pub fn new(module_path: ModulePath) -> Self {
        Self {
            module_path,
            symbols: HashMap::new(),
            model_tests: Vec::new(),
            dependency_tests: HashMap::new(),
            external_imports: Vec::new(),
            section_notes: HashMap::new(),
            section_items: HashMap::new(),
            dependencies: HashSet::new(),
            parameter_dependencies: HashMap::new(),
        }
    }

    pub fn add_symbol(&mut self, name: Identifier, symbol: Symbol) {
        self.symbols.insert(name, symbol);
    }

    pub fn add_model_test(&mut self, test: ast::Test) -> TestIndex {
        let test_index = self.model_tests.len();
        self.model_tests.push(test);
        TestIndex::new(test_index)
    }

    pub fn add_dependency_test(&mut self, path: ModulePath, inputs: TestInputs) {
        self.dependency_tests.insert(path, inputs);
    }

    pub fn add_external_import(&mut self, external_import: PythonPath) {
        self.external_imports.push(external_import);
    }

    pub fn add_section_note(&mut self, section_label: SectionLabel, note: ast::Note) {
        assert!(
            !self.section_notes.contains_key(&section_label),
            "Section note already exists for section label: {:?}",
            section_label
        );

        self.section_notes.insert(section_label, note);
    }

    pub fn add_section_decl(&mut self, section_label: SectionLabel, section_decl: SectionDecl) {
        let section_items = self.section_items.get_mut(&section_label);
        match section_items {
            Some(section_items) => section_items.push(section_decl),
            None => {
                self.section_items.insert(section_label, vec![section_decl]);
            }
        }
    }

    pub fn add_dependency(&mut self, dependency: Dependency) {
        self.dependencies.insert(dependency);
    }

    pub fn add_parameter_dependencies(
        &mut self,
        parameter: Identifier,
        dependencies: HashSet<ParameterDependency>,
    ) {
        self.parameter_dependencies.insert(parameter, dependencies);
    }

    pub fn symbols(&self) -> &HashMap<Identifier, Symbol> {
        &self.symbols
    }

    pub fn tests(&self) -> &Vec<ast::Test> {
        &self.model_tests
    }

    pub fn into_module(self) -> Module {
        let ModuleBuilder {
            module_path,
            symbols,
            model_tests,
            dependency_tests,
            external_imports,
            section_notes,
            section_items,
            dependencies,
            parameter_dependencies,
        } = self;

        let documentation_map = DocumentationMap::new(section_notes, section_items);

        let tests = Tests::new(model_tests, dependency_tests);
        let symbols = SymbolMap::new(symbols);
        let external_imports = ExternalImportList::new(external_imports);

        let module = Module::new(
            module_path,
            symbols,
            tests,
            external_imports,
            documentation_map,
            dependencies,
            parameter_dependencies,
        );

        module
    }
}

impl From<ModuleBuilder> for Module {
    fn from(builder: ModuleBuilder) -> Self {
        builder.into_module()
    }
}

/// A builder pattern implementation for constructing test inputs.
///
/// Provides a mutable interface for incrementally building test inputs,
/// which are then converted into an immutable TestInputs struct when complete.
///
/// This builder is used internally to simplify test input construction.
pub struct TestInputsBuilder {
    inputs: HashMap<Identifier, ast::Expr>,
}

impl TestInputsBuilder {
    pub fn new() -> Self {
        Self {
            inputs: HashMap::new(),
        }
    }

    pub fn add_input(&mut self, name: Identifier, expr: ast::Expr) {
        self.inputs.insert(name, expr);
    }

    pub fn into_test_inputs(self) -> TestInputs {
        self.into()
    }
}

impl From<TestInputsBuilder> for TestInputs {
    fn from(builder: TestInputsBuilder) -> Self {
        TestInputs::new(builder.inputs)
    }
}
