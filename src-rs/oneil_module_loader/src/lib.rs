// TODO: uncomment this once the library is more organized and stable
// #![warn(missing_docs)]

mod builder;
mod error;
mod loader;
mod util;

use oneil_module::{ModuleCollection, ModulePath};
use std::path::Path;

use crate::{
    error::ModuleErrorCollection,
    util::{ModuleCollectionBuilder, Stack},
};

pub use crate::util::FileLoader;

pub fn load_module<F>(
    module_path: impl AsRef<Path>,
    file_parser: &F,
) -> Result<ModuleCollection, (ModuleCollection, ModuleErrorCollection<F::ParseError>)>
where
    F: FileLoader,
{
    let module_path = ModulePath::new(module_path.as_ref().to_path_buf());
    let mut module_stack = Stack::new();
    let module_collection = ModuleCollectionBuilder::new(vec![module_path.clone()]);
    let module_errors = ModuleErrorCollection::new();

    let (module_collection, module_errors) = loader::load_module(
        module_path,
        &mut module_stack,
        module_collection,
        module_errors,
        file_parser,
    );

    let module_collection = module_collection.into_module_collection();
    if module_errors.is_empty() {
        Ok(module_collection)
    } else {
        Err((module_collection, module_errors))
    }
}

pub fn load_module_list<F>(
    module_paths: Vec<impl AsRef<Path>>,
    file_parser: &F,
) -> Result<ModuleCollection, (ModuleCollection, ModuleErrorCollection<F::ParseError>)>
where
    F: FileLoader,
{
    let module_paths: Vec<_> = module_paths
        .into_iter()
        .map(|p| ModulePath::new(p.as_ref().to_path_buf()))
        .collect();
    let module_collection = ModuleCollectionBuilder::new(module_paths.clone());
    let module_errors = ModuleErrorCollection::new();

    let (module_collection, module_errors) = module_paths.into_iter().fold(
        (module_collection, module_errors),
        |(module_collection, module_errors), module_path| {
            let mut module_stack = Stack::new();
            let (module_collection, module_errors) = loader::load_module(
                module_path,
                &mut module_stack,
                module_collection,
                module_errors,
                file_parser,
            );
            (module_collection, module_errors)
        },
    );

    let module_collection = module_collection.into_module_collection();
    if module_errors.is_empty() {
        Ok(module_collection)
    } else {
        Err((module_collection, module_errors))
    }
}
