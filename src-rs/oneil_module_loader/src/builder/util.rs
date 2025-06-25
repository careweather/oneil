use std::collections::{HashMap, HashSet};

use oneil_ast as ast;
use oneil_module::{
    Dependency, DocumentationMap, ExternalImportList, Identifier, Module, ModulePath, PythonPath,
    SectionDecl, SectionLabel, Symbol, SymbolMap, TestIndex, TestInputs, Tests,
};

/// An internal struct for making module-building easier with an API that is
/// tailored to the needs of this module
pub struct ModuleBuilder {
    module_path: ModulePath,
    symbols: SymbolMap,
    tests: Tests,
    external_imports: ExternalImportList,
    section_notes: HashMap<SectionLabel, ast::Note>,
    section_items: HashMap<SectionLabel, Vec<SectionDecl>>,
    dependencies: HashSet<Dependency>,
}

impl ModuleBuilder {
    pub fn new(module_path: ModulePath) -> Self {
        Self {
            module_path,
            symbols: SymbolMap::new(),
            tests: Tests::new(),
            external_imports: ExternalImportList::new(),
            section_notes: HashMap::new(),
            section_items: HashMap::new(),
            dependencies: HashSet::new(),
        }
    }

    pub fn add_symbol(&mut self, name: Identifier, symbol: Symbol) {
        self.symbols.add_symbol(name, symbol);
    }

    pub fn add_model_test(&mut self, test: ast::Test) -> TestIndex {
        self.tests.add_model_test(test)
    }

    pub fn add_dependency_test(&mut self, path: ModulePath, inputs: TestInputs) {
        self.tests.add_dependency_test(path, inputs);
    }

    pub fn add_external_import(&mut self, external_import: PythonPath) {
        self.external_imports.add_import(external_import);
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

    pub fn into_module(self) -> Module {
        self.into()
    }
}

impl From<ModuleBuilder> for Module {
    fn from(builder: ModuleBuilder) -> Self {
        let ModuleBuilder {
            module_path,
            symbols,
            tests,
            external_imports,
            section_notes,
            section_items,
            dependencies,
        } = builder;

        let documentation_map = DocumentationMap::new(section_notes, section_items);

        Module::new(
            module_path,
            symbols,
            tests,
            external_imports,
            documentation_map,
            dependencies,
        )
    }
}
