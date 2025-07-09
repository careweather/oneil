use std::collections::{HashMap, HashSet};

use oneil_module::reference::{ModulePath, PythonPath};

use crate::{
    FileLoader, error::resolution::ImportResolutionError, util::builder::ModuleCollectionBuilder,
};

/// Validates a list of Python import declarations for a given module.
///
/// This function processes import declarations from an Oneil module and validates
/// that the referenced Python files exist. It uses a functional approach with `fold`
/// to accumulate results across all imports.
///
/// # Arguments
///
/// * `module_path` - The path of the module containing the imports
/// * `builder` - A builder for constructing the module collection
/// * `imports` - A vector of import declarations to validate
/// * `file_loader` - A file loader implementation for validating Python imports
///
/// # Returns
///
/// A tuple containing:
/// * `HashSet<PythonPath>` - Successfully validated Python import paths
/// * `HashMap<PythonPath, ImportResolutionError>` - Failed imports with their errors
/// * `ModuleCollectionBuilder<F::ParseError, F::PythonError>` - Updated builder
///
/// # Notes
///
/// - Each import path is converted to a Python path relative to the module's location
/// - Successful imports are added to the returned set of valid Python imports
/// - Failed imports are recorded in both the error map and the builder
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
            eprintln!("{:?}: {:?}", python_path, result);
            match result {
                Ok(()) => {
                    python_imports.insert(python_path);
                    (python_imports, import_resolution_errors, builder)
                }
                Err(error) => {
                    builder.add_import_error(python_path.clone(), error);
                    import_resolution_errors.insert(python_path, ImportResolutionError::new());
                    (python_imports, import_resolution_errors, builder)
                }
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use oneil_ast::declaration::Import;

    use super::*;
    use crate::test::TestPythonValidator;

    fn get_module_path() -> ModulePath {
        ModulePath::new(PathBuf::from("test_module"))
    }

    fn get_empty_builder() -> ModuleCollectionBuilder<(), ()> {
        ModuleCollectionBuilder::new(HashSet::new())
    }

    #[test]
    fn test_validate_imports_empty_list() {
        let file_loader = TestPythonValidator::validate_all();
        let module_path = get_module_path();
        let builder = get_empty_builder();
        let imports = vec![];

        let (valid_imports, errors, _builder) =
            validate_imports(&module_path, builder, imports, &file_loader);

        assert!(valid_imports.is_empty());
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_imports_single_valid_import() {
        let file_loader = TestPythonValidator::validate_all();
        let module_path = get_module_path();
        let builder = get_empty_builder();
        let imports = vec![Import {
            path: "my_python".to_string(),
        }];

        let (valid_imports, errors, _builder) =
            validate_imports(&module_path, builder, imports, &file_loader);

        assert_eq!(valid_imports.len(), 1);
        assert!(valid_imports.contains(&PythonPath::new(PathBuf::from("my_python"))));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_imports_single_invalid_import() {
        let file_loader = TestPythonValidator::validate_none();
        let module_path = get_module_path();
        let builder = get_empty_builder();
        let imports = vec![Import {
            path: "nonexistent".to_string(),
        }];

        let (valid_imports, errors, _builder) =
            validate_imports(&module_path, builder, imports, &file_loader);

        assert!(valid_imports.is_empty());
        assert_eq!(errors.len(), 1);
        assert!(errors.contains_key(&PythonPath::new(PathBuf::from("nonexistent"))));
    }

    #[test]
    fn test_validate_imports_mixed_valid_and_invalid() {
        let file_loader = TestPythonValidator::validate_some(vec!["my_python.py".into()]);
        let module_path = get_module_path();
        let builder = get_empty_builder();
        let imports = vec![
            Import {
                path: "my_python".to_string(),
            },
            Import {
                path: "nonexistent".to_string(),
            },
        ];

        let (valid_imports, errors, _builder) =
            validate_imports(&module_path, builder, imports, &file_loader);

        eprintln!("valid_imports: {:?}", valid_imports);
        eprintln!("errors: {:?}", errors);
        assert_eq!(valid_imports.len(), 1);
        assert!(valid_imports.contains(&PythonPath::new(PathBuf::from("my_python"))));
        assert_eq!(errors.len(), 1);
        assert!(errors.contains_key(&PythonPath::new(PathBuf::from("nonexistent"))));
    }

    #[test]
    fn test_validate_imports_multiple_valid_imports() {
        let file_loader = TestPythonValidator::validate_all();
        let module_path = get_module_path();
        let builder = get_empty_builder();
        let imports = vec![
            Import {
                path: "my_python1".to_string(),
            },
            Import {
                path: "my_python2".to_string(),
            },
            Import {
                path: "my_python3".to_string(),
            },
        ];

        let (valid_imports, errors, _builder) =
            validate_imports(&module_path, builder, imports, &file_loader);

        assert_eq!(valid_imports.len(), 3);
        assert!(valid_imports.contains(&PythonPath::new(PathBuf::from("my_python1"))));
        assert!(valid_imports.contains(&PythonPath::new(PathBuf::from("my_python2"))));
        assert!(valid_imports.contains(&PythonPath::new(PathBuf::from("my_python3"))));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_imports_all_invalid() {
        let file_loader = TestPythonValidator::validate_none();
        let module_path = get_module_path();
        let builder = get_empty_builder();
        let imports = vec![
            Import {
                path: "nonexistent1".to_string(),
            },
            Import {
                path: "nonexistent2".to_string(),
            },
        ];

        let (valid_imports, errors, _builder) =
            validate_imports(&module_path, builder, imports, &file_loader);

        assert!(valid_imports.is_empty());
        assert_eq!(errors.len(), 2);
        assert!(errors.contains_key(&PythonPath::new(PathBuf::from("nonexistent1"))));
        assert!(errors.contains_key(&PythonPath::new(PathBuf::from("nonexistent2"))));
    }

    #[test]
    fn test_validate_imports_builder_error_tracking() {
        let file_loader = TestPythonValidator::validate_none();
        let module_path = get_module_path();
        let builder = get_empty_builder();
        let imports = vec![Import {
            path: "nonexistent".to_string(),
        }];

        let (_valid_imports, _errors, builder) =
            validate_imports(&module_path, builder, imports, &file_loader);

        assert!(
            builder
                .get_imports_with_errors()
                .contains(&PythonPath::new(PathBuf::from("nonexistent")))
        );
    }

    #[test]
    fn test_validate_imports_path_conversion() {
        let file_loader = TestPythonValidator::validate_some(vec!["subdir/my_python.py".into()]);
        let module_path = ModulePath::new(PathBuf::from("subdir/test_module"));
        let builder = get_empty_builder();
        let imports = vec![Import {
            path: "my_python".to_string(),
        }];

        let (valid_imports, _errors, _builder) =
            validate_imports(&module_path, builder, imports, &file_loader);

        // The import should be converted to a Python path relative to the module
        assert_eq!(valid_imports.len(), 1);
        assert!(valid_imports.contains(&PythonPath::new(PathBuf::from("subdir/my_python"))));
    }
}
