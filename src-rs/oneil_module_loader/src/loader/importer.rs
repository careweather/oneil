use std::collections::HashSet;

use oneil_module::reference::{ModulePath, PythonPath};

use crate::{FileLoader, util::builder::ModuleCollectionBuilder};

pub fn validate_imports<F>(
    module_path: &ModulePath,
    builder: ModuleCollectionBuilder,
    imports: Vec<oneil_ast::declaration::Import>,
    file_loader: &F,
) -> (HashSet<PythonPath>, ModuleCollectionBuilder)
where
    F: FileLoader,
{
    // TODO: check for duplicate imports
    imports.into_iter().fold(
        (HashSet::new(), builder),
        |(mut python_imports, mut builder), import| {
            let python_path = module_path.get_sibling_path(&import.path);
            let python_path = PythonPath::new(python_path);

            let result = file_loader.validate_python_import(&python_path);
            match result {
                Ok(()) => {
                    python_imports.insert(python_path);
                    (python_imports, builder)
                }
                Err(error) => {
                    builder.add_error(module_path.clone(), todo!("pass error along"));
                    (python_imports, builder)
                }
            }
        },
    )
}
