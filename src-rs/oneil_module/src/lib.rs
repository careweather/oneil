use std::{collections::HashMap, path::PathBuf};

use oneil_ast as ast;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Ident(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ModulePath(PathBuf);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PythonPath(PathBuf);

#[derive(Debug, Clone, PartialEq)]
struct PythonModule(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SectionLabel(String);

#[derive(Debug, Clone, PartialEq)]
struct ModuleReference {
    path: ModulePath,
    subcomponents: Vec<Ident>,
}

#[derive(Debug, Clone, PartialEq)]
struct TestIndex(usize);

#[derive(Debug, Clone, PartialEq)]
struct ImportIndex(usize);

#[derive(Debug, Clone, PartialEq)]
enum SectionItem {
    Test(TestIndex),
    Parameter(Ident),
    InternalImport(Ident),
    ExternalImport(ImportIndex),
}

#[derive(Debug, Clone, PartialEq)]
struct SectionMap {
    top_section: SectionData,
    sub_sections: HashMap<SectionLabel, SectionData>,
}

#[derive(Debug, Clone, PartialEq)]
struct SectionData {
    note: Option<ast::Note>,
    items: Vec<SectionItem>,
}

#[derive(Debug, Clone, PartialEq)]
enum Symbol {
    Parameter(ast::Parameter),
    Import(ModuleReference),
}

#[derive(Debug, Clone, PartialEq)]
struct Module {
    symbols: HashMap<Ident, Symbol>,
    tests: Vec<ast::Test>,
    external_imports: Vec<PythonPath>,
    sections: SectionMap,
}

#[derive(Debug, Clone, PartialEq)]
struct ModuleCollection {
    initial_module: Option<ModulePath>,
    modules: HashMap<ModulePath, Module>,
    python_imports: HashMap<PythonPath, PythonModule>,
}
