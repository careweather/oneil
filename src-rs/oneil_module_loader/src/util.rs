use std::{collections::HashMap, path::Path};

use oneil_ast as ast;
use oneil_module::{Module, ModuleCollection, ModulePath};

pub trait FileLoader {
    type ParseError;
    fn parse_ast(&self, path: impl AsRef<Path>) -> Result<ast::Model, Self::ParseError>;
    fn file_exists(&self, path: impl AsRef<Path>) -> bool;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stack<T>
where
    T: PartialEq + Clone,
{
    stack: Vec<T>,
}

impl<T> Stack<T>
where
    T: PartialEq + Clone,
{
    pub fn new() -> Self {
        Self { stack: vec![] }
    }

    pub fn push(&mut self, path: T) {
        self.stack.push(path);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.stack.pop()
    }

    pub fn check_for_cyclical_dependency(&self, path: &T) -> Option<Vec<T>> {
        // Get the index of the last occurence of the path in the stack, if any exists
        let last_index = self.stack.iter().rposition(|p| p == path);

        match last_index {
            Some(index) => {
                let cyclical_deps = self.stack[index..].to_vec();
                Some(cyclical_deps)
            }
            None => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleCollectionBuilder {
    initial_modules: Vec<ModulePath>,
    modules: HashMap<ModulePath, Module>,
}

impl ModuleCollectionBuilder {
    pub fn new(initial_modules: Vec<ModulePath>) -> Self {
        Self {
            initial_modules,
            modules: HashMap::new(),
        }
    }

    pub fn add_module(&mut self, module_path: &ModulePath, module: Module) {
        self.modules.insert(module_path.clone(), module);
    }

    pub fn has_loaded_for(&self, module_path: &ModulePath) -> bool {
        self.modules.contains_key(module_path)
    }

    pub fn into_module_collection(self) -> ModuleCollection {
        let ModuleCollectionBuilder {
            initial_modules,
            modules,
        } = self;

        ModuleCollection::new(initial_modules, modules)
    }
}

impl From<ModuleCollectionBuilder> for ModuleCollection {
    fn from(builder: ModuleCollectionBuilder) -> Self {
        builder.into_module_collection()
    }
}
