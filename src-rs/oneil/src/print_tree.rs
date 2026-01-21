use oneil_eval::output::{
    dependency::{DependencyTreeValue, RequiresTreeValue},
    tree::Tree,
};

pub struct TreePrintConfig {
    pub recursive: bool,
    pub depth: Option<usize>,
    pub partial: bool,
}

pub(crate) fn print_requires_tree(
    requires_tree: &Tree<RequiresTreeValue>,
    tree_print_config: &TreePrintConfig,
) {
    todo!()
}

pub(crate) fn print_dependency_tree(
    dependency_tree: &Tree<DependencyTreeValue>,
    tree_print_config: &TreePrintConfig,
) {
    todo!()
}
