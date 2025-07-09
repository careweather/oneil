//! Module structures and collections for the Oneil programming language.
//!
//! This module defines the core data structures for representing Oneil modules,
//! including their parameters, tests, submodels, and Python imports. Modules
//! are the primary organizational unit in Oneil, containing all the components
//! needed to define a model or submodel.

use std::collections::{HashMap, HashSet};

use crate::{
    parameter::{Parameter, ParameterCollection},
    reference::{Identifier, ModulePath, PythonPath},
    test::{ModelTest, SubmodelTest, TestIndex},
};

/// Represents a single Oneil module containing parameters, tests, submodels, and imports.
///
/// A module is the fundamental building block in Oneil, representing either a complete
/// model or a reusable submodel. Each module can contain:
///
/// - **Parameters**: Named values with expressions and constraints
/// - **Tests**: Validation rules for the module's behavior
/// - **Submodels**: References to other modules that this module depends on
/// - **Python Imports**: External Python modules that provide additional functionality
///
/// Modules are immutable by design, following functional programming principles.
#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    python_imports: HashSet<PythonPath>,
    submodels: HashMap<Identifier, ModulePath>,
    parameters: ParameterCollection,
    model_tests: HashMap<TestIndex, ModelTest>,
    submodel_tests: Vec<SubmodelTest>,
}

impl Module {
    /// Creates a new module with the specified components.
    ///
    /// # Arguments
    ///
    /// * `python_imports` - Set of Python modules to import
    /// * `submodels` - Mapping of submodel identifiers to their module paths
    /// * `parameters` - Collection of parameters defined in this module
    /// * `model_tests` - Tests for the entire model
    /// * `submodel_tests` - Tests for individual submodels
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_module::{module::Module, parameter::ParameterCollection};
    /// use std::collections::{HashMap, HashSet};
    ///
    /// let module = Module::new(
    ///     HashSet::new(), // no Python imports
    ///     HashMap::new(),  // no submodels
    ///     ParameterCollection::new(HashMap::new()),
    ///     HashMap::new(),  // no model tests
    ///     Vec::new(),      // no submodel tests
    /// );
    /// ```
    pub fn new(
        python_imports: HashSet<PythonPath>,
        submodels: HashMap<Identifier, ModulePath>,
        parameters: ParameterCollection,
        model_tests: HashMap<TestIndex, ModelTest>,
        submodel_tests: Vec<SubmodelTest>,
    ) -> Self {
        Self {
            python_imports,
            submodels,
            parameters,
            model_tests,
            submodel_tests,
        }
    }

    /// Returns a reference to the set of Python imports for this module.
    ///
    /// Python imports allow modules to use external Python functionality
    /// for complex calculations or data processing.
    pub fn get_python_imports(&self) -> &HashSet<PythonPath> {
        &self.python_imports
    }

    /// Looks up a submodel by its identifier.
    ///
    /// Returns `Some(ModulePath)` if the submodel exists, `None` otherwise.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The identifier of the submodel to look up
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_module::{module::Module, reference::{Identifier, ModulePath}, parameter::ParameterCollection};
    /// use std::collections::{HashMap, HashSet};
    ///
    /// let mut submodels = HashMap::new();
    /// submodels.insert(Identifier::new("sub"), ModulePath::new("submodule"));
    ///
    /// let module = Module::new(
    ///     HashSet::new(),
    ///     submodels,
    ///     ParameterCollection::new(HashMap::new()),
    ///     HashMap::new(),
    ///     Vec::new(),
    /// );
    ///
    /// assert!(module.get_submodel(&Identifier::new("sub")).is_some());
    /// assert!(module.get_submodel(&Identifier::new("nonexistent")).is_none());
    /// ```
    pub fn get_submodel(&self, identifier: &Identifier) -> Option<&ModulePath> {
        self.submodels.get(identifier)
    }

    /// Looks up a parameter by its identifier.
    ///
    /// Returns `Some(Parameter)` if the parameter exists, `None` otherwise.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The identifier of the parameter to look up
    pub fn get_parameter(&self, identifier: &Identifier) -> Option<&Parameter> {
        self.parameters.get(identifier)
    }

    /// Returns a reference to all model tests in this module.
    ///
    /// Model tests validate the behavior of the entire module and are
    /// indexed by test indices for easy lookup.
    pub fn get_model_tests(&self) -> &HashMap<TestIndex, ModelTest> {
        &self.model_tests
    }

    /// Returns a reference to all submodel tests in this module.
    ///
    /// Submodel tests validate the behavior of individual submodels
    /// and are stored in a vector since they don't need indexed access.
    pub fn get_submodel_tests(&self) -> &Vec<SubmodelTest> {
        &self.submodel_tests
    }

    /// Checks if this module is empty (contains no components).
    ///
    /// A module is considered empty if it has no Python imports, submodels,
    /// parameters, model tests, or submodel tests.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_module::{module::Module, parameter::ParameterCollection};
    /// use std::collections::{HashMap, HashSet};
    ///
    /// let empty_module = Module::new(
    ///     HashSet::new(),
    ///     HashMap::new(),
    ///     ParameterCollection::new(HashMap::new()),
    ///     HashMap::new(),
    ///     Vec::new(),
    /// );
    ///
    /// assert!(empty_module.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.python_imports.is_empty()
            && self.submodels.is_empty()
            && self.parameters.is_empty()
            && self.model_tests.is_empty()
            && self.submodel_tests.is_empty()
    }
}

/// A collection of modules that can be managed together.
///
/// `ModuleCollection` provides a way to organize and manage multiple modules,
/// particularly useful for handling module dependencies and resolving imports.
/// It maintains a set of initial modules (entry points) and a mapping of all
/// available modules.
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleCollection {
    initial_modules: HashSet<ModulePath>,
    modules: HashMap<ModulePath, Module>,
}

impl ModuleCollection {
    /// Creates a new module collection with the specified initial modules and module mapping.
    ///
    /// # Arguments
    ///
    /// * `initial_modules` - Set of module paths that serve as entry points
    /// * `modules` - Mapping of module paths to their corresponding modules
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_module::{module::{ModuleCollection, Module}, reference::ModulePath, parameter::ParameterCollection};
    /// use std::collections::{HashMap, HashSet};
    ///
    /// let mut initial_modules = HashSet::new();
    /// initial_modules.insert(ModulePath::new("main"));
    ///
    /// let mut modules = HashMap::new();
    /// modules.insert(ModulePath::new("main"), Module::new(
    ///     HashSet::new(),
    ///     HashMap::new(),
    ///     ParameterCollection::new(HashMap::new()),
    ///     HashMap::new(),
    ///     Vec::new(),
    /// ));
    ///
    /// let collection = ModuleCollection::new(initial_modules, modules);
    /// ```
    pub fn new(initial_modules: HashSet<ModulePath>, modules: HashMap<ModulePath, Module>) -> Self {
        Self {
            initial_modules,
            modules,
        }
    }

    /// Returns all Python imports from all modules in the collection.
    ///
    /// This method aggregates Python imports from all modules, which is useful
    /// for dependency analysis and ensuring all required Python modules are available.
    ///
    /// # Returns
    ///
    /// A set of references to all Python paths imported across all modules.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_module::{module::{ModuleCollection, Module}, reference::{ModulePath, PythonPath}, parameter::ParameterCollection};
    /// use std::collections::{HashMap, HashSet};
    /// use std::path::PathBuf;
    ///
    /// let mut initial_modules = HashSet::new();
    /// initial_modules.insert(ModulePath::new("main"));
    ///
    /// let mut python_imports = HashSet::new();
    /// python_imports.insert(PythonPath::new(PathBuf::from("math")));
    ///
    /// let mut modules = HashMap::new();
    /// modules.insert(ModulePath::new("main"), Module::new(
    ///     python_imports,
    ///     HashMap::new(),
    ///     ParameterCollection::new(HashMap::new()),
    ///     HashMap::new(),
    ///     Vec::new(),
    /// ));
    ///
    /// let collection = ModuleCollection::new(initial_modules, modules);
    /// let imports = collection.get_python_imports();
    /// assert_eq!(imports.len(), 1);
    /// ```
    pub fn get_python_imports(&self) -> HashSet<&PythonPath> {
        self.modules
            .values()
            .flat_map(|module| module.python_imports.iter())
            .collect()
    }
}
