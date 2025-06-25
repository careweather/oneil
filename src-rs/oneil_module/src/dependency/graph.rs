use std::{collections::HashMap, hash::Hash};

/// A graph of dependencies between nodes
///
/// This is a directed graph where each node has a list of dependencies and a
/// list of dependents. The dependencies are the nodes that the current node
/// depends on. The dependents are the nodes that depend on the current node.
///
/// This is used to track the dependencies between modules and parameters.
///
/// This structure constructs a `dependent_nodes` edge list from a `dependencies`
/// edge list in order to reduce the amount of time required to get the dependent
/// nodes for a given node.
#[derive(Debug, Clone, PartialEq)]
pub struct DependencyGraph<T>
where
    T: Eq + Hash + Clone,
{
    dependencies: HashMap<T, Vec<T>>,
    dependent_nodes: HashMap<T, Vec<T>>,
}

impl<T> DependencyGraph<T>
where
    T: Eq + Hash + Clone,
{
    /// Create a new dependency graph
    pub fn new(dependencies: HashMap<T, Vec<T>>) -> Self {
        let dependent_nodes = Self::construct_dependent_nodes(&dependencies);
        Self {
            dependencies,
            dependent_nodes,
        }
    }

    fn construct_dependent_nodes(dependencies: &HashMap<T, Vec<T>>) -> HashMap<T, Vec<T>> {
        let mut dependent_nodes = HashMap::new();
        for (node, dependencies) in dependencies {
            for dependency in dependencies {
                dependent_nodes
                    .entry(dependency.clone())
                    .or_insert(vec![])
                    .push(node.clone());
            }
        }
        dependent_nodes
    }

    /// Get the dependencies map
    pub fn dependencies(&self) -> &HashMap<T, Vec<T>> {
        &self.dependencies
    }

    /// Get the dependent nodes map
    pub fn dependent_nodes(&self) -> &HashMap<T, Vec<T>> {
        &self.dependent_nodes
    }

    /// Get the dependencies of a node
    pub fn get_dependencies_for(&self, node: &T) -> Option<&Vec<T>> {
        self.dependencies.get(node)
    }

    /// Get the dependents of a node
    pub fn get_dependent_nodes_for(&self, node: &T) -> Option<&Vec<T>> {
        self.dependent_nodes.get(node)
    }
}
