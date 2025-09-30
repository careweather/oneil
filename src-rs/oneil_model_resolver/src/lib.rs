#![cfg_attr(doc, doc = include_str!("../README.md"))]
//! Model resolver for the Oneil programming language

use std::{collections::HashSet, path::Path};

use oneil_ir as ir;

use crate::util::{Stack, builder::ModelCollectionBuilder};

pub mod error;
mod resolver;
mod util;

#[cfg(test)]
mod test;

pub use crate::error::collection::ModelErrorMap;
pub use crate::util::FileLoader;
pub use crate::util::builtin_ref::BuiltinRef;

type LoadModelOk = Box<ir::ModelCollection>;
type LoadModelErr<Ps, Py> = Box<(ir::ModelCollection, ModelErrorMap<Ps, Py>)>;

/// Loads a single model and all its dependencies.
///
/// This is the main entry point for loading a Oneil model. It loads the specified model
/// and all of its dependencies, returning either a complete `ModelCollection` or a tuple
/// containing a partial collection and any errors that occurred during loading.
pub fn load_model<F>(
    model_path: impl AsRef<Path>,
    builtin_ref: &impl BuiltinRef,
    file_parser: &F,
) -> Result<LoadModelOk, LoadModelErr<F::ParseError, F::PythonError>>
where
    F: FileLoader,
{
    load_model_list(&[model_path], builtin_ref, file_parser)
}

/// Loads multiple models and all their dependencies.
pub fn load_model_list<F>(
    model_paths: &[impl AsRef<Path>],
    builtin_ref: &impl BuiltinRef,
    file_parser: &F,
) -> Result<LoadModelOk, LoadModelErr<F::ParseError, F::PythonError>>
where
    F: FileLoader,
{
    let initial_model_paths: HashSet<_> = model_paths
        .iter()
        .map(AsRef::as_ref)
        .map(ir::ModelPath::new)
        .collect();

    let builder = ModelCollectionBuilder::new(initial_model_paths);

    let builder = model_paths.iter().fold(builder, |builder, model_path| {
        let model_path = ir::ModelPath::new(model_path.as_ref());
        let mut load_stack = Stack::new();

        resolver::load_model(
            model_path,
            builder,
            builtin_ref,
            &mut load_stack,
            file_parser,
        )
    });

    builder.try_into().map(Box::new).map_err(Box::new)
}
