use std::collections::{HashMap, HashSet};

use oneil_module::reference::{ModulePath, PythonPath};

use crate::{
    FileLoader, error::resolution::ImportResolutionError, util::builder::ModuleCollectionBuilder,
};

pub fn validate_imports<F>(
    module_path: &ModulePath,
    builder: ModuleCollectionBuilder<F::ParseError, F::PythonError>,
    imports: Vec<oneil_ast::declaration::Import>,
    file_loader: &F,
) -> (
    HashSet<PythonPath>,
    HashMap<PythonPath, ImportResolutionError>,
    ModuleCollectionBuilder<F::ParseError, F::PythonError>,
)
where
    F: FileLoader,
{
    // TODO: check for duplicate imports
    imports.into_iter().fold(
        (HashSet::new(), HashMap::new(), builder),
        |(mut python_imports, mut import_resolution_errors, mut builder), import| {
            let python_path = module_path.get_sibling_path(&import.path);
            let python_path = PythonPath::new(python_path);

            let result = file_loader.validate_python_import(&python_path);
            match result {
                Ok(()) => {
                    python_imports.insert(python_path);
                    (python_imports, import_resolution_errors, builder)
                }
                Err(error) => {
                    builder.add_python_error(python_path.clone(), error);
                    import_resolution_errors.insert(python_path, ImportResolutionError::new());
                    (python_imports, import_resolution_errors, builder)
                }
            }
        },
    )
}
