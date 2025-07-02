use oneil_module::reference::ModulePath;

pub mod collection;

#[derive(Debug, Clone, PartialEq)]
pub enum LoadError<Ps, Py> {
    ModuleError(ModuleError),
    ParseError(Ps),
    PythonError(Py),
}

impl<Ps, Py> LoadError<Ps, Py> {
    pub fn module_circular_dependency(circular_dependency: Vec<ModulePath>) -> Self {
        Self::ModuleError(ModuleError::CircularDependency(circular_dependency))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModuleError {
    CircularDependency(Vec<ModulePath>),
}
