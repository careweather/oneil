use std::{collections::HashMap, path::PathBuf};

use crate::ast;

struct Ident(String);

struct ModulePath(PathBuf);

struct PythonPath(PathBuf);

struct SectionLabel(String);

struct Reference {
    path: ModulePath,
    subcomponents: Vec<Ident>,
}

struct Module {
    ast: ast::Model,
    tests: Vec<(Option<SectionLabel>, ast::Test)>,
    parameters: Vec<(Option<SectionLabel>, ast::Parameter)>,
    import_map: HashMap<Ident, Reference>,
    python_imports: Vec<PythonPath>,
}

struct ModuleCollection {
    initial_module: Option<ModulePath>,
    modules: HashMap<ModulePath, Module>,
}
