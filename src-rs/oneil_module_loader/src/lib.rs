use std::{collections::HashSet, path::Path};

use oneil_module::{module::ModuleCollection, reference::ModulePath};

use crate::{
    error::collection::ModuleErrorMap,
    util::{Stack, builder::ModuleCollectionBuilder},
};

mod error;
mod loader;
mod util;

#[cfg(test)]
mod test;

pub use crate::util::FileLoader;

pub fn load_module<F>(
    module_path: impl AsRef<Path>,
    file_parser: &F,
) -> Result<
    ModuleCollection,
    (
        ModuleCollection,
        ModuleErrorMap<F::ParseError, F::PythonError>,
    ),
>
where
    F: FileLoader,
{
    load_module_list(&[module_path], file_parser)
}

pub fn load_module_list<F>(
    module_paths: &[impl AsRef<Path>],
    file_parser: &F,
) -> Result<
    ModuleCollection,
    (
        ModuleCollection,
        ModuleErrorMap<F::ParseError, F::PythonError>,
    ),
>
where
    F: FileLoader,
{
    let initial_module_paths: HashSet<_> = module_paths
        .iter()
        .map(|p| ModulePath::new(p.as_ref().to_path_buf()))
        .collect();

    let builder = ModuleCollectionBuilder::new(initial_module_paths);

    let builder = module_paths.iter().fold(builder, |builder, module_path| {
        let module_path = ModulePath::new(module_path.as_ref().to_path_buf());
        let mut load_stack = Stack::new();

        loader::load_module(module_path, builder, &mut load_stack, file_parser)
    });

    builder.try_into()
}
