//! A dependency graph for the results of evaluating Oneil models.

use indexmap::{IndexMap, IndexSet};
use oneil_output::{BuiltinDependency, DependencySet, ExternalDependency, ParameterDependency};
use oneil_shared::{
    EvalInstanceKey,
    symbols::{ParameterName, ReferenceName, TestIndex},
};

/// A dependency graph for the results of evaluating Oneil models.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyGraph {
    depends_on: IndexMap<EvalInstanceKey, IndexMap<ParameterName, DependencySet>>,
    referenced_by: IndexMap<EvalInstanceKey, IndexMap<ParameterName, ReferenceSet>>,
}

impl DependencyGraph {
    /// Creates a new dependency graph.
    #[must_use]
    pub fn new() -> Self {
        Self {
            depends_on: IndexMap::new(),
            referenced_by: IndexMap::new(),
        }
    }

    /// Adds a builtin dependency to the graph.
    pub fn add_depends_on_builtin(
        &mut self,
        param_key: EvalInstanceKey,
        param_name: ParameterName,
        dependency: BuiltinDependency,
    ) {
        self.depends_on
            .entry(param_key)
            .or_default()
            .entry(param_name)
            .or_default()
            .builtin_dependencies
            .insert(dependency);
    }

    /// Adds a parameter dependency to the graph.
    pub fn add_depends_on_parameter(
        &mut self,
        param_key: EvalInstanceKey,
        param_name: ParameterName,
        dependency: ParameterDependency,
    ) {
        self.depends_on
            .entry(param_key.clone())
            .or_default()
            .entry(param_name.clone())
            .or_default()
            .parameter_dependencies
            .insert(dependency.clone());

        let ParameterDependency {
            parameter_name: dependency_parameter_name,
        } = dependency;

        let reference = ParameterReference {
            parameter_name: param_name,
        };

        self.referenced_by
            .entry(param_key)
            .or_default()
            .entry(dependency_parameter_name)
            .or_default()
            .parameter
            .insert(reference);
    }

    /// Adds an external dependency to the graph.
    pub fn add_depends_on_external(
        &mut self,
        param_key: EvalInstanceKey,
        param_name: ParameterName,
        dependency: ExternalDependency,
    ) {
        self.depends_on
            .entry(param_key.clone())
            .or_default()
            .entry(param_name.clone())
            .or_default()
            .external_dependencies
            .insert(dependency.clone());

        let ExternalDependency {
            instance_key: dependency_instance_key,
            reference_name: dependency_reference_name,
            parameter_name: dependency_parameter_name,
        } = dependency;

        let reference = ExternalParameterReference {
            instance_key: param_key,
            parameter_name: param_name,
            using_reference_name: dependency_reference_name,
        };

        self.referenced_by
            .entry(dependency_instance_key)
            .or_default()
            .entry(dependency_parameter_name)
            .or_default()
            .external_parameter
            .insert(reference);
    }

    /// Records that a test depends on another parameter in the same model, and the reverse edge.
    pub fn add_test_depends_on_parameter(
        &mut self,
        instance_key: EvalInstanceKey,
        test_index: TestIndex,
        dependency: ParameterDependency,
    ) {
        let ParameterDependency {
            parameter_name: dependency_parameter_name,
        } = dependency;

        let reference = TestReference { test_index };

        self.referenced_by
            .entry(instance_key)
            .or_default()
            .entry(dependency_parameter_name)
            .or_default()
            .test
            .insert(reference);
    }

    /// Records that a test depends on a parameter in another model, and the reverse edge.
    pub fn add_test_depends_on_external(
        &mut self,
        instance_key: EvalInstanceKey,
        test_index: TestIndex,
        dependency: ExternalDependency,
    ) {
        let ExternalDependency {
            instance_key: dependency_instance_key,
            reference_name: dependency_reference_name,
            parameter_name: dependency_parameter_name,
        } = dependency;

        self.referenced_by
            .entry(dependency_instance_key)
            .or_default()
            .entry(dependency_parameter_name)
            .or_default()
            .external_test
            .insert(ExternalTestReference {
                instance_key,
                test_index,
                using_reference_name: dependency_reference_name,
            });
    }

    /// Returns the parameters that a given parameter depends on.
    #[must_use]
    pub fn dependents(
        &self,
        instance_key: &EvalInstanceKey,
        parameter_name: &ParameterName,
    ) -> Option<&DependencySet> {
        let model = self.depends_on.get(instance_key)?;
        model.get(parameter_name)
    }

    /// Returns the parameters that reference a given parameter.
    #[must_use]
    pub fn references(
        &self,
        instance_key: &EvalInstanceKey,
        parameter_name: &ParameterName,
    ) -> Option<&ReferenceSet> {
        let model = self.referenced_by.get(instance_key)?;
        model.get(parameter_name)
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// A set of parameters that reference a given parameter.
///
/// This structure tracks which other parameters or external models reference
/// a given parameter. This is the reverse mapping of dependencies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceSet {
    /// Parameters within the same model that reference this parameter.
    pub parameter: IndexSet<ParameterReference>,
    /// External models that reference this parameter.
    pub external_parameter: IndexSet<ExternalParameterReference>,
    /// Tests that reference this parameter.
    pub test: IndexSet<TestReference>,
    /// Tests in other models that reference this parameter through a reference import.
    pub external_test: IndexSet<ExternalTestReference>,
}

impl ReferenceSet {
    /// Creates a new empty reference set.
    #[must_use]
    pub fn new() -> Self {
        Self {
            parameter: IndexSet::new(),
            external_parameter: IndexSet::new(),
            test: IndexSet::new(),
            external_test: IndexSet::new(),
        }
    }
}

impl Default for ReferenceSet {
    fn default() -> Self {
        Self::new()
    }
}

/// A reference from another parameter within the same model.
///
/// This represents the reverse relationship of a `ParameterDependency`:
/// it indicates that another parameter in the same model references this parameter.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParameterReference {
    /// The name of the parameter that references this parameter.
    pub parameter_name: ParameterName,
}

/// A reference from an external model.
///
/// This represents the reverse relationship of an `ExternalDependency`:
/// it indicates that a parameter in another model references this parameter.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternalParameterReference {
    /// The evaluated model instance that references this parameter.
    pub instance_key: EvalInstanceKey,
    /// The name of the parameter in the external model that references this parameter.
    pub parameter_name: ParameterName,
    /// The reference name used by the external model to access this model.
    pub using_reference_name: ReferenceName,
}

/// A reference from a test.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TestReference {
    /// The index of the test that references this parameter.
    pub test_index: TestIndex,
}

/// A reference from a test in another model, via a reference import.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternalTestReference {
    /// The evaluated model instance that contains the test.
    pub instance_key: EvalInstanceKey,
    /// The index of the test that references this parameter.
    pub test_index: TestIndex,
    /// The reference name the test's model uses for the model that defines this parameter.
    pub using_reference_name: ReferenceName,
}
