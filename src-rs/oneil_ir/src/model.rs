//! Model structures and collections for the Oneil programming language.
//!
//! This module defines the core data structures for representing Oneil models
//! as an intermediate representation (IR), including their parameters, tests, submodels,
//! and Python imports. Models are the primary organizational unit in Oneil,
//! containing all the components needed to define a model or submodel.

use std::collections::{HashMap, HashSet};

use crate::{
    parameter::{Parameter, ParameterCollection},
    reference::{Identifier, ModelPath, PythonPath},
    test::{SubmodelTest, Test, TestIndex},
};

/// Represents a single Oneil model containing parameters, tests, submodels, and imports.
///
/// A model is the fundamental building block in Oneil, representing either a complete
/// model or a reusable submodel. Each model can contain:
///
/// - **Parameters**: Named values with expressions and constraints
/// - **Tests**: Validation rules for the model's behavior
/// - **Submodels**: References to other models that this model depends on
/// - **Python Imports**: External Python modules that provide additional functionality
///
/// Models are immutable by design, following functional programming principles.
#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    python_imports: HashSet<PythonPath>,
    submodels: HashMap<Identifier, ModelPath>,
    parameters: ParameterCollection,
    tests: HashMap<TestIndex, Test>,
    submodel_tests: Vec<SubmodelTest>,
}

impl Model {
    /// Creates a new model with the specified components.
    ///
    /// # Arguments
    ///
    /// * `python_imports` - Set of Python modules to import
    /// * `submodels` - Mapping of submodel identifiers to their model paths
    /// * `parameters` - Collection of parameters defined in this model
    /// * `tests` - Tests for the entire model
    /// * `submodel_tests` - Tests for individual submodels
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{model::Model, parameter::ParameterCollection};
    /// use std::collections::{HashMap, HashSet};
    ///
    /// let model = Model::new(
    ///     HashSet::new(), // no Python imports
    ///     HashMap::new(),  // no submodels
    ///     ParameterCollection::new(HashMap::new()),
    ///     HashMap::new(),  // no tests
    ///     Vec::new(),      // no submodel tests
    /// );
    /// ```
    pub fn new(
        python_imports: HashSet<PythonPath>,
        submodels: HashMap<Identifier, ModelPath>,
        parameters: ParameterCollection,
        tests: HashMap<TestIndex, Test>,
        submodel_tests: Vec<SubmodelTest>,
    ) -> Self {
        Self {
            python_imports,
            submodels,
            parameters,
            tests,
            submodel_tests,
        }
    }

    /// Returns a reference to the set of Python imports for this model.
    ///
    /// Python imports allow models to use external Python functionality
    /// for complex calculations or data processing.
    pub fn get_python_imports(&self) -> &HashSet<PythonPath> {
        &self.python_imports
    }

    /// Looks up a submodel by its identifier.
    ///
    /// Returns `Some(ModelPath)` if the submodel exists, `None` otherwise.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The identifier of the submodel to look up
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{model::Model, reference::{Identifier, ModelPath}, parameter::ParameterCollection};
    /// use std::collections::{HashMap, HashSet};
    ///
    /// let mut submodels = HashMap::new();
    /// submodels.insert(Identifier::new("sub"), ModelPath::new("submodel"));
    ///
    /// let model = Model::new(
    ///     HashSet::new(),
    ///     submodels,
    ///     ParameterCollection::new(HashMap::new()),
    ///     HashMap::new(),
    ///     Vec::new(),
    /// );
    ///
    /// assert!(model.get_submodel(&Identifier::new("sub")).is_some());
    /// assert!(model.get_submodel(&Identifier::new("nonexistent")).is_none());
    /// ```
    pub fn get_submodel(&self, identifier: &Identifier) -> Option<&ModelPath> {
        self.submodels.get(identifier)
    }

    /// Returns a reference to all submodels in this model.
    ///
    /// Submodels represent nested models that are used as components within the current model.
    /// They are stored in a map where the key is the identifier used to reference the submodel
    /// and the value is the path to the actual model definition.
    ///
    /// # Returns
    ///
    /// A reference to the `HashMap` mapping submodel identifiers to their model paths.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{model::Model, reference::{Identifier, ModelPath}, parameter::ParameterCollection};
    /// use std::collections::{HashMap, HashSet};
    ///
    /// let mut submodels = HashMap::new();
    /// submodels.insert(Identifier::new("sub"), ModelPath::new("submodel"));
    ///
    /// let model = Model::new(
    ///     HashSet::new(),
    ///     submodels,
    ///     ParameterCollection::new(HashMap::new()),
    ///     HashMap::new(),
    ///     Vec::new(),
    /// );
    ///
    /// let all_submodels = model.get_submodels();
    /// assert_eq!(all_submodels.len(), 1);
    /// ```
    pub fn get_submodels(&self) -> &HashMap<Identifier, ModelPath> {
        &self.submodels
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

    /// Returns a reference to all tests in this model.
    ///
    /// Tests validate the behavior of the entire model and are
    /// indexed by test indices for easy lookup.
    pub fn get_tests(&self) -> &HashMap<TestIndex, Test> {
        &self.tests
    }

    /// Returns a reference to all submodel tests in this model.
    ///
    /// Submodel tests validate the behavior of individual submodels
    /// and are stored in a vector since they don't need indexed access.
    pub fn get_submodel_tests(&self) -> &Vec<SubmodelTest> {
        &self.submodel_tests
    }

    /// Checks if this model is empty (contains no components).
    ///
    /// A model is considered empty if it has no Python imports, submodels,
    /// parameters, tests, or submodel tests.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{model::Model, parameter::ParameterCollection};
    /// use std::collections::{HashMap, HashSet};
    ///
    /// let empty_model = Model::new(
    ///     HashSet::new(),
    ///     HashMap::new(),
    ///     ParameterCollection::new(HashMap::new()),
    ///     HashMap::new(),
    ///     Vec::new(),
    /// );
    ///
    /// assert!(empty_model.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.python_imports.is_empty()
            && self.submodels.is_empty()
            && self.parameters.is_empty()
            && self.tests.is_empty()
            && self.submodel_tests.is_empty()
    }

    /// Returns a set of model paths that this model depends on through submodels.
    ///
    /// This method collects all unique model paths referenced by submodels in this model.
    /// It's useful for dependency analysis and determining the order of model evaluation.
    ///
    /// # Returns
    ///
    /// A HashSet containing references to all ModelPaths that this model depends on.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{model::Model, reference::{ModelPath, Identifier}, parameter::ParameterCollection};
    /// use std::collections::{HashMap, HashSet};
    ///
    /// let mut submodels = HashMap::new();
    /// submodels.insert(Identifier::new("sub1"), ModelPath::new("submodel1"));
    /// submodels.insert(Identifier::new("sub2"), ModelPath::new("submodel2"));
    ///
    /// let model = Model::new(
    ///     HashSet::new(),
    ///     submodels,
    ///     ParameterCollection::new(HashMap::new()),
    ///     HashMap::new(),
    ///     Vec::new(),
    /// );
    ///
    /// let dependencies = model.get_model_dependencies();
    /// assert_eq!(dependencies.len(), 2);
    /// assert!(dependencies.contains(&ModelPath::new("submodel1")));
    /// assert!(dependencies.contains(&ModelPath::new("submodel2")));
    /// ```
    pub fn get_model_dependencies(&self) -> HashSet<&ModelPath> {
        let mut dependencies = HashSet::new();

        for submodel_path in self.submodels.values() {
            dependencies.insert(submodel_path);
        }

        dependencies
    }

    /// Returns a list of parameter identifiers in evaluation order.
    ///
    /// This method performs a depth-first traversal of the parameter dependency graph
    /// to determine the correct order for evaluating parameters. The returned order
    /// ensures that all dependencies of a parameter are evaluated before the parameter
    /// itself.
    ///
    /// # Returns
    ///
    /// A vector of references to parameter Identifiers in evaluation order.
    ///
    /// Note that this is a partial order, so no order guarantees are made aside
    /// from the fact that all dependencies of a parameter are evaluated before
    /// the parameter itself.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{model::Model, reference::Identifier, parameter::{Parameter, ParameterCollection, ParameterValue, Limits}, expr::{Expr, Literal}, debug_info::TraceLevel};
    /// use std::collections::{HashMap, HashSet};
    ///
    /// let mut deps = HashSet::new();
    /// deps.insert(Identifier::new("radius"));
    ///
    /// let mut params = HashMap::new();
    /// params.insert(Identifier::new("radius"), Parameter::new(
    ///     HashSet::new(),
    ///     Identifier::new("radius"),
    ///     ParameterValue::simple(Expr::literal(Literal::number(5.0)), None),
    ///     Limits::default(),
    ///     false,
    ///     TraceLevel::None,
    /// ));
    /// params.insert(Identifier::new("area"), Parameter::new(
    ///     deps,
    ///     Identifier::new("area"),
    ///     ParameterValue::simple(Expr::literal(Literal::number(78.5)), None),
    ///     Limits::default(),
    ///     false,
    ///     TraceLevel::None,
    /// ));
    ///
    /// let model = Model::new(
    ///     HashSet::new(),
    ///     HashMap::new(),
    ///     ParameterCollection::new(params),
    ///     HashMap::new(),
    ///     Vec::new(),
    /// );
    ///
    /// let order = model.get_parameter_evaluation_order();
    /// assert_eq!(order.len(), 2);
    /// assert_eq!(order[0].identifier(), &Identifier::new("radius")); // dependency first
    /// assert_eq!(order[1].identifier(), &Identifier::new("area")); // then dependent parameter
    /// ```
    pub fn get_parameter_evaluation_order(&self) -> Vec<&Parameter> {
        let (evaluation_order, _) = self.parameters.iter().fold(
            (Vec::new(), HashSet::new()),
            |(evaluation_order, visited), (_, parameter)| {
                self.get_parameter_evaluation_order_recursive(parameter, evaluation_order, visited)
            },
        );

        evaluation_order
    }

    fn get_parameter_evaluation_order_recursive<'a>(
        &'a self,
        parameter: &'a Parameter,
        evaluation_order: Vec<&'a Parameter>,
        mut visited: HashSet<&'a Identifier>,
    ) -> (Vec<&'a Parameter>, HashSet<&'a Identifier>) {
        if visited.contains(&parameter.identifier()) {
            return (evaluation_order, visited);
        }

        visited.insert(parameter.identifier());

        let (mut evaluation_order, final_visited) = parameter.dependencies().iter().fold(
            (evaluation_order, visited),
            |(evaluation_order, visited), dependency| {
                let dependency_parameter = self.get_parameter(dependency).expect("should exist");
                self.get_parameter_evaluation_order_recursive(
                    dependency_parameter,
                    evaluation_order,
                    visited,
                )
            },
        );

        evaluation_order.push(parameter);

        (evaluation_order, final_visited)
    }
}

/// A collection of models that can be managed together.
///
/// `ModelCollection` provides a way to organize and manage multiple models,
/// particularly useful for handling model dependencies and resolving imports.
/// It maintains a set of initial models (entry points) and a mapping of all
/// available models.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelCollection {
    initial_models: HashSet<ModelPath>,
    models: HashMap<ModelPath, Model>,
}

impl ModelCollection {
    /// Creates a new model collection with the specified initial models and model mapping.
    ///
    /// # Arguments
    ///
    /// * `initial_models` - Set of model paths that serve as entry points
    /// * `models` - Mapping of model paths to their corresponding models
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{model::{ModelCollection, Model}, reference::ModelPath, parameter::ParameterCollection};
    /// use std::collections::{HashMap, HashSet};
    ///
    /// let mut initial_models = HashSet::new();
    /// initial_models.insert(ModelPath::new("main"));
    ///
    /// let mut models = HashMap::new();
    /// models.insert(ModelPath::new("main"), Model::new(
    ///     HashSet::new(),
    ///     HashMap::new(),
    ///     ParameterCollection::new(HashMap::new()),
    ///     HashMap::new(),
    ///     Vec::new(),
    /// ));
    ///
    /// let collection = ModelCollection::new(initial_models, models);
    /// ```
    pub fn new(initial_models: HashSet<ModelPath>, models: HashMap<ModelPath, Model>) -> Self {
        Self {
            initial_models,
            models,
        }
    }

    /// Returns a reference to the initial models in this collection.
    ///
    /// This method provides access to the set of model paths that serve as entry points
    /// in the model collection.
    ///
    /// # Returns
    ///
    /// A reference to the `HashSet` containing the initial model paths.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{model::{ModelCollection, Model}, reference::ModelPath, parameter::ParameterCollection};
    /// use std::collections::{HashMap, HashSet};
    ///
    /// let mut initial_models = HashSet::new();
    /// initial_models.insert(ModelPath::new("main"));
    ///
    /// let mut models = HashMap::new();
    /// models.insert(ModelPath::new("main"), Model::new(
    ///     HashSet::new(),
    ///     HashMap::new(),
    ///     ParameterCollection::new(HashMap::new()),
    ///     HashMap::new(),
    ///     Vec::new(),
    /// ));
    ///
    /// let collection = ModelCollection::new(initial_models, models);
    /// let initial = collection.initial_models();
    /// assert_eq!(initial.len(), 1);
    /// ```
    pub fn initial_models(&self) -> &HashSet<ModelPath> {
        &self.initial_models
    }

    /// Returns a reference to the models in this collection.
    ///
    /// This method provides access to the map of model paths to their corresponding models
    /// in the collection.
    ///
    /// # Returns
    ///
    /// A reference to the `HashMap` containing the model paths and their associated models.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{model::{ModelCollection, Model}, reference::ModelPath, parameter::ParameterCollection};
    /// use std::collections::{HashMap, HashSet};
    ///
    /// let mut initial_models = HashSet::new();
    /// initial_models.insert(ModelPath::new("main"));
    ///
    /// let mut models = HashMap::new();
    /// models.insert(ModelPath::new("main"), Model::new(
    ///     HashSet::new(),
    ///     HashMap::new(),
    ///     ParameterCollection::new(HashMap::new()),
    ///     HashMap::new(),
    ///     Vec::new(),
    /// ));
    ///
    /// let collection = ModelCollection::new(initial_models, models);
    /// let model_map = collection.models();
    /// assert_eq!(model_map.len(), 1);
    /// ```
    pub fn models(&self) -> &HashMap<ModelPath, Model> {
        &self.models
    }

    /// Returns all Python imports from all modelss in the collection.
    ///
    /// This method aggregates Python imports from all models, which is useful
    /// for dependency analysis and ensuring all required Python modules are available.
    ///
    /// # Returns
    ///
    /// A set of references to all Python paths imported across all models.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{model::{ModelCollection, Model}, reference::{ModelPath, PythonPath}, parameter::ParameterCollection};
    /// use std::collections::{HashMap, HashSet};
    /// use std::path::PathBuf;
    ///
    /// let mut initial_models = HashSet::new();
    /// initial_models.insert(ModelPath::new("main"));
    ///
    /// let mut python_imports = HashSet::new();
    /// python_imports.insert(PythonPath::new(PathBuf::from("math")));
    ///
    /// let mut models = HashMap::new();
    /// models.insert(ModelPath::new("main"), Model::new(
    ///     python_imports,
    ///     HashMap::new(),
    ///     ParameterCollection::new(HashMap::new()),
    ///     HashMap::new(),
    ///     Vec::new(),
    /// ));
    ///
    /// let collection = ModelCollection::new(initial_models, models);
    /// let imports = collection.get_python_imports();
    /// assert_eq!(imports.len(), 1);
    /// ```
    pub fn get_python_imports(&self) -> HashSet<&PythonPath> {
        self.models
            .values()
            .flat_map(|model| model.python_imports.iter())
            .collect()
    }

    /// Returns a topologically sorted list of model paths in evaluation order.
    ///
    /// This method performs a depth-first traversal of the model dependency graph,
    /// starting from the initial models, to determine the correct order for evaluating
    /// models. The returned order ensures that all dependencies of a model are evaluated
    /// before the model itself.
    ///
    /// # Returns
    ///
    /// A vector of references to ModelPaths in topological order.
    ///
    /// Note that this is a partial order, so no order guarantees are made aside
    /// from the fact that all dependencies of a model are evaluated before
    /// the model itself.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::{model::{ModelCollection, Model}, reference::{ModelPath, Identifier}, parameter::ParameterCollection};
    /// use std::collections::{HashMap, HashSet};
    ///
    /// let mut initial_models = HashSet::new();
    /// initial_models.insert(ModelPath::new("main"));
    ///
    /// let mut submodels = HashMap::new();
    /// submodels.insert(Identifier::new("sub"), ModelPath::new("sub"));
    ///
    /// let mut models = HashMap::new();
    /// models.insert(ModelPath::new("main"), Model::new(
    ///     HashSet::new(),
    ///     submodels,
    ///     ParameterCollection::new(HashMap::new()),
    ///     HashMap::new(),
    ///     Vec::new(),
    /// ));
    /// models.insert(ModelPath::new("sub"), Model::new(
    ///     HashSet::new(),
    ///     HashMap::new(),
    ///     ParameterCollection::new(HashMap::new()),
    ///     HashMap::new(),
    ///     Vec::new(),
    /// ));
    ///
    /// let collection = ModelCollection::new(initial_models, models);
    /// let order = collection.get_model_evaluation_order();
    /// assert_eq!(order.len(), 2);
    /// assert_eq!(order[0], &ModelPath::new("sub")); // dependency first
    /// assert_eq!(order[1], &ModelPath::new("main")); // then main model
    /// ```
    pub fn get_model_evaluation_order(&self) -> Vec<&ModelPath> {
        let (evaluation_order, _) = self.initial_models().iter().fold(
            (Vec::new(), HashSet::new()),
            |(evaluation_order, visited), model_path| {
                self.model_evaluation_order_recursive(model_path, evaluation_order, visited)
            },
        );
        evaluation_order
    }

    fn model_evaluation_order_recursive<'a>(
        &'a self,
        model_path: &'a ModelPath,
        evaluation_order: Vec<&'a ModelPath>,
        mut visited: HashSet<&'a ModelPath>,
    ) -> (Vec<&'a ModelPath>, HashSet<&'a ModelPath>) {
        if visited.contains(model_path) {
            return (Vec::new(), visited);
        }

        visited.insert(model_path);

        let model = self.models.get(model_path).expect("model should exist");

        let (mut evaluation_order, final_visited) = model.get_model_dependencies().iter().fold(
            (evaluation_order, visited),
            |(evaluation_order, visited), dependency| {
                self.model_evaluation_order_recursive(dependency, evaluation_order, visited)
            },
        );

        evaluation_order.push(model_path);

        (evaluation_order, final_visited)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        debug_info::TraceLevel,
        expr::{Expr, Literal},
        parameter::{Limits, Parameter, ParameterCollection, ParameterValue},
        reference::{Identifier, ModelPath, PythonPath},
        test::{SubmodelTest, SubmodelTestInputs, Test, TestIndex},
    };
    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;

    #[test]
    fn test_model_is_empty_when_empty() {
        let empty_model = Model::new(
            HashSet::new(),
            HashMap::new(),
            ParameterCollection::new(HashMap::new()),
            HashMap::new(),
            Vec::new(),
        );
        assert!(empty_model.is_empty());
    }

    #[test]
    fn test_model_is_not_empty_with_python_imports() {
        let mut python_imports = HashSet::new();
        python_imports.insert(PythonPath::new(PathBuf::from("math")));
        let model_with_imports = Model::new(
            python_imports,
            HashMap::new(),
            ParameterCollection::new(HashMap::new()),
            HashMap::new(),
            Vec::new(),
        );
        assert!(!model_with_imports.is_empty());
    }

    #[test]
    fn test_model_is_not_empty_with_submodels() {
        let mut submodels = HashMap::new();
        submodels.insert(Identifier::new("sub"), ModelPath::new("submodel"));
        let model_with_submodels = Model::new(
            HashSet::new(),
            submodels,
            ParameterCollection::new(HashMap::new()),
            HashMap::new(),
            Vec::new(),
        );
        assert!(!model_with_submodels.is_empty());
    }

    #[test]
    fn test_model_is_not_empty_with_parameters() {
        let mut params = HashMap::new();
        params.insert(
            Identifier::new("radius"),
            Parameter::new(
                HashSet::new(),
                Identifier::new("radius"),
                ParameterValue::simple(Expr::literal(Literal::number(5.0)), None),
                Limits::default(),
                false,
                TraceLevel::None,
            ),
        );
        let model_with_params = Model::new(
            HashSet::new(),
            HashMap::new(),
            ParameterCollection::new(params),
            HashMap::new(),
            Vec::new(),
        );
        assert!(!model_with_params.is_empty());
    }

    #[test]
    fn test_model_is_not_empty_with_tests() {
        let mut tests = HashMap::new();
        tests.insert(
            TestIndex::new(0),
            Test::new(
                TraceLevel::None,
                HashSet::new(),
                Expr::literal(Literal::number(1.0)),
            ),
        );
        let model_with_tests = Model::new(
            HashSet::new(),
            HashMap::new(),
            ParameterCollection::new(HashMap::new()),
            tests,
            Vec::new(),
        );
        assert!(!model_with_tests.is_empty());
    }

    #[test]
    fn test_model_is_not_empty_with_submodel_tests() {
        let submodel_tests = vec![SubmodelTest::new(
            Identifier::new("sub"),
            SubmodelTestInputs::new(HashMap::new()),
        )];
        let model_with_submodel_tests = Model::new(
            HashSet::new(),
            HashMap::new(),
            ParameterCollection::new(HashMap::new()),
            HashMap::new(),
            submodel_tests,
        );
        assert!(!model_with_submodel_tests.is_empty());
    }

    #[test]
    fn test_model_get_model_dependencies_empty() {
        let empty_model = Model::new(
            HashSet::new(),
            HashMap::new(),
            ParameterCollection::new(HashMap::new()),
            HashMap::new(),
            Vec::new(),
        );
        let dependencies = empty_model.get_model_dependencies();
        assert!(dependencies.is_empty());
    }

    #[test]
    fn test_model_get_model_dependencies_single() {
        let mut submodels = HashMap::new();
        submodels.insert(Identifier::new("sub1"), ModelPath::new("submodel1"));
        let model_with_one_submodel = Model::new(
            HashSet::new(),
            submodels,
            ParameterCollection::new(HashMap::new()),
            HashMap::new(),
            Vec::new(),
        );
        let dependencies = model_with_one_submodel.get_model_dependencies();
        assert_eq!(dependencies.len(), 1);
        assert!(dependencies.contains(&ModelPath::new("submodel1")));
    }

    #[test]
    fn test_model_get_model_dependencies_multiple() {
        let mut submodels = HashMap::new();
        submodels.insert(Identifier::new("sub1"), ModelPath::new("submodel1"));
        submodels.insert(Identifier::new("sub2"), ModelPath::new("submodel2"));
        submodels.insert(Identifier::new("sub3"), ModelPath::new("submodel3"));
        let model_with_multiple_submodels = Model::new(
            HashSet::new(),
            submodels,
            ParameterCollection::new(HashMap::new()),
            HashMap::new(),
            Vec::new(),
        );
        let dependencies = model_with_multiple_submodels.get_model_dependencies();
        assert_eq!(dependencies.len(), 3);
        assert!(dependencies.contains(&ModelPath::new("submodel1")));
        assert!(dependencies.contains(&ModelPath::new("submodel2")));
        assert!(dependencies.contains(&ModelPath::new("submodel3")));
    }

    #[test]
    fn test_model_get_model_dependencies_deduplicates() {
        let mut submodels = HashMap::new();
        submodels.insert(Identifier::new("sub1"), ModelPath::new("shared_model"));
        submodels.insert(Identifier::new("sub2"), ModelPath::new("shared_model"));
        let model_with_duplicate_paths = Model::new(
            HashSet::new(),
            submodels,
            ParameterCollection::new(HashMap::new()),
            HashMap::new(),
            Vec::new(),
        );
        let dependencies = model_with_duplicate_paths.get_model_dependencies();
        assert_eq!(dependencies.len(), 1); // HashSet deduplicates
        assert!(dependencies.contains(&ModelPath::new("shared_model")));
    }

    #[test]
    fn test_model_get_parameter_evaluation_order_empty() {
        let empty_model = Model::new(
            HashSet::new(),
            HashMap::new(),
            ParameterCollection::new(HashMap::new()),
            HashMap::new(),
            Vec::new(),
        );
        let order = empty_model.get_parameter_evaluation_order();
        assert!(order.is_empty());
    }

    #[test]
    fn test_model_get_parameter_evaluation_order_single() {
        let mut params = HashMap::new();
        params.insert(
            Identifier::new("radius"),
            Parameter::new(
                HashSet::new(),
                Identifier::new("radius"),
                ParameterValue::simple(Expr::literal(Literal::number(5.0)), None),
                Limits::default(),
                false,
                TraceLevel::None,
            ),
        );
        let model_with_one_param = Model::new(
            HashSet::new(),
            HashMap::new(),
            ParameterCollection::new(params),
            HashMap::new(),
            Vec::new(),
        );
        let order = model_with_one_param.get_parameter_evaluation_order();
        assert_eq!(order.len(), 1);
        assert_eq!(order[0].identifier(), &Identifier::new("radius"));
    }

    #[test]
    fn test_model_get_parameter_evaluation_order_linear_chain() {
        let mut params = HashMap::new();
        params.insert(
            Identifier::new("radius"),
            Parameter::new(
                HashSet::new(),
                Identifier::new("radius"),
                ParameterValue::simple(Expr::literal(Literal::number(5.0)), None),
                Limits::default(),
                false,
                TraceLevel::None,
            ),
        );
        let mut area_deps = HashSet::new();
        area_deps.insert(Identifier::new("radius"));
        params.insert(
            Identifier::new("area"),
            Parameter::new(
                area_deps,
                Identifier::new("area"),
                ParameterValue::simple(Expr::literal(Literal::number(78.5)), None),
                Limits::default(),
                false,
                TraceLevel::None,
            ),
        );
        let model_with_linear_chain = Model::new(
            HashSet::new(),
            HashMap::new(),
            ParameterCollection::new(params),
            HashMap::new(),
            Vec::new(),
        );
        let order = model_with_linear_chain.get_parameter_evaluation_order();
        assert_eq!(order.len(), 2);
        assert_eq!(order[0].identifier(), &Identifier::new("radius")); // dependency first
        assert_eq!(order[1].identifier(), &Identifier::new("area")); // then dependent parameter
    }

    #[test]
    fn test_model_get_parameter_evaluation_order_complex_graph() {
        let mut params = HashMap::new();
        // Independent parameters
        params.insert(
            Identifier::new("base"),
            Parameter::new(
                HashSet::new(),
                Identifier::new("base"),
                ParameterValue::simple(Expr::literal(Literal::number(10.0)), None),
                Limits::default(),
                false,
                TraceLevel::None,
            ),
        );
        params.insert(
            Identifier::new("height"),
            Parameter::new(
                HashSet::new(),
                Identifier::new("height"),
                ParameterValue::simple(Expr::literal(Literal::number(5.0)), None),
                Limits::default(),
                false,
                TraceLevel::None,
            ),
        );
        // Parameters depending on base
        let mut area_deps = HashSet::new();
        area_deps.insert(Identifier::new("base"));
        area_deps.insert(Identifier::new("height"));
        params.insert(
            Identifier::new("area"),
            Parameter::new(
                area_deps,
                Identifier::new("area"),
                ParameterValue::simple(Expr::literal(Literal::number(25.0)), None),
                Limits::default(),
                false,
                TraceLevel::None,
            ),
        );
        // Parameter depending on area
        let mut volume_deps = HashSet::new();
        volume_deps.insert(Identifier::new("area"));
        params.insert(
            Identifier::new("volume"),
            Parameter::new(
                volume_deps,
                Identifier::new("volume"),
                ParameterValue::simple(Expr::literal(Literal::number(125.0)), None),
                Limits::default(),
                false,
                TraceLevel::None,
            ),
        );
        let model_with_complex_deps = Model::new(
            HashSet::new(),
            HashMap::new(),
            ParameterCollection::new(params),
            HashMap::new(),
            Vec::new(),
        );
        let order = model_with_complex_deps.get_parameter_evaluation_order();
        assert_eq!(order.len(), 4);

        // Check that dependencies come before dependents
        let base_idx = order
            .iter()
            .position(|&param| param.identifier() == &Identifier::new("base"))
            .unwrap();
        let height_idx = order
            .iter()
            .position(|&param| param.identifier() == &Identifier::new("height"))
            .unwrap();
        let area_idx = order
            .iter()
            .position(|&param| param.identifier() == &Identifier::new("area"))
            .unwrap();
        let volume_idx = order
            .iter()
            .position(|&param| param.identifier() == &Identifier::new("volume"))
            .unwrap();

        assert!(base_idx < area_idx);
        assert!(height_idx < area_idx);
        assert!(area_idx < volume_idx);
    }

    #[test]
    fn test_model_collection_get_python_imports_empty() {
        let collection = ModelCollection::new(HashSet::new(), HashMap::new());
        let imports = collection.get_python_imports();
        assert!(imports.is_empty());
    }

    #[test]
    fn test_model_collection_get_python_imports_single_model_no_imports() {
        let mut initial_models = HashSet::new();
        initial_models.insert(ModelPath::new("main"));
        let mut models = HashMap::new();
        models.insert(
            ModelPath::new("main"),
            Model::new(
                HashSet::new(),
                HashMap::new(),
                ParameterCollection::new(HashMap::new()),
                HashMap::new(),
                Vec::new(),
            ),
        );
        let collection = ModelCollection::new(initial_models, models);
        let imports = collection.get_python_imports();
        assert!(imports.is_empty());
    }

    #[test]
    fn test_model_collection_get_python_imports_single_model_with_imports() {
        let mut initial_models = HashSet::new();
        initial_models.insert(ModelPath::new("main"));
        let mut python_imports = HashSet::new();
        python_imports.insert(PythonPath::new(PathBuf::from("math")));
        python_imports.insert(PythonPath::new(PathBuf::from("numpy")));
        let mut models = HashMap::new();
        models.insert(
            ModelPath::new("main"),
            Model::new(
                python_imports,
                HashMap::new(),
                ParameterCollection::new(HashMap::new()),
                HashMap::new(),
                Vec::new(),
            ),
        );
        let collection = ModelCollection::new(initial_models, models);
        let imports = collection.get_python_imports();
        assert_eq!(imports.len(), 2);
        assert!(
            imports
                .iter()
                .any(|&path| path.as_ref() == PathBuf::from("math.py").as_path())
        );
        assert!(
            imports
                .iter()
                .any(|&path| path.as_ref() == PathBuf::from("numpy.py").as_path())
        );
    }

    #[test]
    fn test_model_collection_get_python_imports_multiple_models_deduplicates() {
        let mut initial_models = HashSet::new();
        initial_models.insert(ModelPath::new("main"));
        initial_models.insert(ModelPath::new("sub"));

        let mut python_imports1 = HashSet::new();
        python_imports1.insert(PythonPath::new(PathBuf::from("math")));
        python_imports1.insert(PythonPath::new(PathBuf::from("numpy")));

        let mut python_imports2 = HashSet::new();
        python_imports2.insert(PythonPath::new(PathBuf::from("math"))); // overlapping
        python_imports2.insert(PythonPath::new(PathBuf::from("scipy")));

        let mut models = HashMap::new();
        models.insert(
            ModelPath::new("main"),
            Model::new(
                python_imports1,
                HashMap::new(),
                ParameterCollection::new(HashMap::new()),
                HashMap::new(),
                Vec::new(),
            ),
        );
        models.insert(
            ModelPath::new("sub"),
            Model::new(
                python_imports2,
                HashMap::new(),
                ParameterCollection::new(HashMap::new()),
                HashMap::new(),
                Vec::new(),
            ),
        );
        let collection = ModelCollection::new(initial_models, models);
        let imports = collection.get_python_imports();
        assert_eq!(imports.len(), 3); // math, numpy, scipy (deduplicated)
        assert!(
            imports
                .iter()
                .any(|&path| path.as_ref() == PathBuf::from("math.py").as_path())
        );
        assert!(
            imports
                .iter()
                .any(|&path| path.as_ref() == PathBuf::from("numpy.py").as_path())
        );
        assert!(
            imports
                .iter()
                .any(|&path| path.as_ref() == PathBuf::from("scipy.py").as_path())
        );
    }

    #[test]
    fn test_model_collection_model_evaluation_order_empty() {
        let collection = ModelCollection::new(HashSet::new(), HashMap::new());
        let order = collection.get_model_evaluation_order();
        assert!(order.is_empty());
    }

    #[test]
    fn test_model_collection_model_evaluation_order_single_model() {
        let mut initial_models = HashSet::new();
        initial_models.insert(ModelPath::new("main"));
        let mut models = HashMap::new();
        models.insert(
            ModelPath::new("main"),
            Model::new(
                HashSet::new(),
                HashMap::new(),
                ParameterCollection::new(HashMap::new()),
                HashMap::new(),
                Vec::new(),
            ),
        );
        let collection = ModelCollection::new(initial_models, models);
        let order = collection.get_model_evaluation_order();
        assert_eq!(order.len(), 1);
        assert_eq!(order[0], &ModelPath::new("main"));
    }

    #[test]
    fn test_model_collection_model_evaluation_order_linear_chain() {
        let mut initial_models = HashSet::new();
        initial_models.insert(ModelPath::new("main"));

        let mut submodels = HashMap::new();
        submodels.insert(Identifier::new("sub"), ModelPath::new("sub"));

        let mut models = HashMap::new();
        models.insert(
            ModelPath::new("main"),
            Model::new(
                HashSet::new(),
                submodels,
                ParameterCollection::new(HashMap::new()),
                HashMap::new(),
                Vec::new(),
            ),
        );
        models.insert(
            ModelPath::new("sub"),
            Model::new(
                HashSet::new(),
                HashMap::new(),
                ParameterCollection::new(HashMap::new()),
                HashMap::new(),
                Vec::new(),
            ),
        );
        let collection = ModelCollection::new(initial_models, models);
        let order = collection.get_model_evaluation_order();
        assert_eq!(order.len(), 2);
        assert_eq!(order[0], &ModelPath::new("sub")); // dependency first
        assert_eq!(order[1], &ModelPath::new("main")); // then main model
    }

    #[test]
    fn test_model_collection_model_evaluation_order_complex_graph() {
        let mut initial_models = HashSet::new();
        initial_models.insert(ModelPath::new("main"));

        let mut main_submodels = HashMap::new();
        main_submodels.insert(Identifier::new("sub1"), ModelPath::new("sub1"));
        main_submodels.insert(Identifier::new("sub2"), ModelPath::new("sub2"));

        let mut sub1_submodels = HashMap::new();
        sub1_submodels.insert(Identifier::new("base"), ModelPath::new("base"));

        let mut models = HashMap::new();
        models.insert(
            ModelPath::new("main"),
            Model::new(
                HashSet::new(),
                main_submodels,
                ParameterCollection::new(HashMap::new()),
                HashMap::new(),
                Vec::new(),
            ),
        );
        models.insert(
            ModelPath::new("sub1"),
            Model::new(
                HashSet::new(),
                sub1_submodels,
                ParameterCollection::new(HashMap::new()),
                HashMap::new(),
                Vec::new(),
            ),
        );
        models.insert(
            ModelPath::new("sub2"),
            Model::new(
                HashSet::new(),
                HashMap::new(),
                ParameterCollection::new(HashMap::new()),
                HashMap::new(),
                Vec::new(),
            ),
        );
        models.insert(
            ModelPath::new("base"),
            Model::new(
                HashSet::new(),
                HashMap::new(),
                ParameterCollection::new(HashMap::new()),
                HashMap::new(),
                Vec::new(),
            ),
        );
        let collection = ModelCollection::new(initial_models, models);
        let order = collection.get_model_evaluation_order();
        assert_eq!(order.len(), 4);

        // Check that dependencies come before dependents
        let base_idx = order
            .iter()
            .position(|&path| path == &ModelPath::new("base"))
            .unwrap();
        let sub1_idx = order
            .iter()
            .position(|&path| path == &ModelPath::new("sub1"))
            .unwrap();
        let sub2_idx = order
            .iter()
            .position(|&path| path == &ModelPath::new("sub2"))
            .unwrap();
        let main_idx = order
            .iter()
            .position(|&path| path == &ModelPath::new("main"))
            .unwrap();

        assert!(base_idx < sub1_idx);
        assert!(sub1_idx < main_idx);
        assert!(sub2_idx < main_idx);
    }

    #[test]
    fn test_model_collection_model_evaluation_order_circular_dependency() {
        let mut initial_models = HashSet::new();
        initial_models.insert(ModelPath::new("circular"));

        let mut circular_submodels = HashMap::new();
        circular_submodels.insert(Identifier::new("self"), ModelPath::new("circular"));

        let mut models = HashMap::new();
        models.insert(
            ModelPath::new("circular"),
            Model::new(
                HashSet::new(),
                circular_submodels,
                ParameterCollection::new(HashMap::new()),
                HashMap::new(),
                Vec::new(),
            ),
        );
        let collection = ModelCollection::new(initial_models, models);
        let order = collection.get_model_evaluation_order();
        // Should still return the model, even with circular dependency
        assert_eq!(order.len(), 1);
        assert_eq!(order[0], &ModelPath::new("circular"));
    }
}
