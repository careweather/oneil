use std::collections::HashSet;

use oneil_module::reference::{ModulePath, PythonPath};

use crate::util::builder::ModuleCollectionBuilder;

pub fn validate_imports(
    module_path: &ModulePath,
    builder: ModuleCollectionBuilder,
    imports: Vec<oneil_ast::declaration::Import>,
) -> (HashSet<PythonPath>, ModuleCollectionBuilder) {
    // TODO: check for duplicate imports
    imports.into_iter().fold(
        (HashSet::new(), builder),
        |(mut python_imports, mut builder), import| {
            let python_path = module_path.get_sibling_path(&import.path);
            let python_path = PythonPath::new(python_path);

            let result = validate_python_import(&python_path);
            match result {
                Ok(()) => {
                    python_imports.insert(python_path);
                    (python_imports, builder)
                }
                Err(error) => {
                    builder.add_error(module_path.clone(), error);
                    (python_imports, builder)
                }
            }
        },
    )
}

fn validate_python_import(python_path: &PythonPath) -> Result<(), ()> {
    todo!()
}
