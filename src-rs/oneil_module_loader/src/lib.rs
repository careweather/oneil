// TODO: uncomment this once the library is more organized and stable
// #![warn(missing_docs)]

mod error;
mod load_module;
mod module_stack;
mod traits;

use oneil_module::{ModuleCollection, ModulePath};
use std::path::Path;

use crate::{error::ModuleErrorCollection, module_stack::ModuleStack};

pub use crate::traits::FileParser;

pub fn load_module<F>(
    module_path: impl AsRef<Path>,
    file_parser: &F,
) -> Result<ModuleCollection, (ModuleCollection, ModuleErrorCollection<F::ParseError>)>
where
    F: FileParser,
{
    let module_path = ModulePath::new(module_path.as_ref().to_path_buf());
    let module_stack = ModuleStack::new();
    let module_collection = ModuleCollection::new(vec![module_path.clone()]);
    let module_errors = ModuleErrorCollection::new();

    let (module_collection, module_errors) = load_module::load_module(
        module_path,
        module_stack,
        module_collection,
        module_errors,
        file_parser,
    );

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
    F: FileParser,
{
    let module_paths: Vec<_> = module_paths
        .into_iter()
        .map(|p| ModulePath::new(p.as_ref().to_path_buf()))
        .collect();
    let module_collection = ModuleCollection::new(module_paths.clone());
    let module_errors = ModuleErrorCollection::new();

    let (module_collection, module_errors) = module_paths.into_iter().fold(
        (module_collection, module_errors),
        |(module_collection, module_errors), module_path| {
            let module_stack = ModuleStack::new();
            let (module_collection, module_errors) = load_module::load_module(
                module_path,
                module_stack,
                module_collection,
                module_errors,
                file_parser,
            );
            (module_collection, module_errors)
        },
    );

    if module_errors.is_empty() {
        Ok(module_collection)
    } else {
        Err((module_collection, module_errors))
    }
}
