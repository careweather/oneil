use std::collections::{HashMap, HashSet};

use oneil_ir::reference::{ModelPath, PythonPath};

use crate::{
    FileLoader, error::resolution::ImportResolutionError, util::builder::ModelCollectionBuilder,
};

/// Validates a list of Python import declarations for a given model.
///
/// This function processes import declarations from an Oneil model and validates
/// that the referenced Python files exist. It uses a functional approach with `fold`
/// to accumulate results across all imports.
///
/// # Arguments
///
/// * `model_path` - The path of the model containing the imports
/// * `builder` - A builder for constructing the model collection
/// * `imports` - A vector of import declarations to validate
/// * `file_loader` - A file loader implementation for validating Python imports
///
/// # Returns
///
/// A tuple containing:
/// * `HashSet<PythonPath>` - Successfully validated Python import paths
/// * `HashMap<PythonPath, ImportResolutionError>` - Failed imports with their errors
/// * `ModelCollectionBuilder<F::ParseError, F::PythonError>` - Updated builder
///
/// # Notes
///
/// - Each import path is converted to a Python path relative to the model's location
/// - Successful imports are added to the returned set of valid Python imports
/// - Failed imports are recorded in both the error map and the builder
pub fn validate_imports<F>(
    model_path: &ModelPath,
    builder: ModelCollectionBuilder<F::ParseError, F::PythonError>,
    imports: Vec<&oneil_ast::declaration::ImportNode>,
    file_loader: &F,
) -> (
    HashSet<PythonPath>,
    HashMap<PythonPath, ImportResolutionError>,
    ModelCollectionBuilder<F::ParseError, F::PythonError>,
)
where
    F: FileLoader,
{
    // TODO: check for duplicate imports
    imports.into_iter().fold(
        (HashSet::new(), HashMap::new(), builder),
        |(mut python_imports, mut import_resolution_errors, mut builder), import| {
            let python_path = model_path.get_sibling_path(&import.path());
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

    use oneil_ast::{
        Span,
        declaration::{Import, ImportNode},
    };

    use super::*;
    use crate::test::TestPythonValidator;

    fn get_model_path() -> ModelPath {
        ModelPath::new(PathBuf::from("test_model"))
    }

    fn get_empty_builder() -> ModelCollectionBuilder<(), ()> {
        ModelCollectionBuilder::new(HashSet::new())
    }

    fn build_import(path: &str) -> ImportNode {
        // for simplicity's sake, we'll use a span that's the length of the path
        let span = Span::new(0, path.len(), 0);
        let import = Import::new(path.to_string());
        ImportNode::new(span, import)
    }

    #[test]
    fn test_validate_imports_empty_list() {
        let file_loader = TestPythonValidator::validate_all();
        let model_path = get_model_path();
        let builder = get_empty_builder();
        let imports = vec![];

        let (valid_imports, errors, _builder) =
            validate_imports(&model_path, builder, imports, &file_loader);

        assert!(valid_imports.is_empty());
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_imports_single_valid_import() {
        let file_loader = TestPythonValidator::validate_all();
        let model_path = get_model_path();
        let builder = get_empty_builder();
        let imports = vec![build_import("my_python")];
        let import_refs = imports.iter().collect();

        let (valid_imports, errors, _builder) =
            validate_imports(&model_path, builder, import_refs, &file_loader);

        assert_eq!(valid_imports.len(), 1);
        assert!(valid_imports.contains(&PythonPath::new(PathBuf::from("my_python"))));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_imports_single_invalid_import() {
        let file_loader = TestPythonValidator::validate_none();
        let model_path = get_model_path();
        let builder = get_empty_builder();
        let imports = vec![build_import("nonexistent")];
        let import_refs = imports.iter().collect();

        let (valid_imports, errors, _builder) =
            validate_imports(&model_path, builder, import_refs, &file_loader);

        assert!(valid_imports.is_empty());
        assert_eq!(errors.len(), 1);
        assert!(errors.contains_key(&PythonPath::new(PathBuf::from("nonexistent"))));
    }

    #[test]
    fn test_validate_imports_mixed_valid_and_invalid() {
        let file_loader = TestPythonValidator::validate_some(vec!["my_python.py".into()]);
        let model_path = get_model_path();
        let builder = get_empty_builder();
        let imports = vec![build_import("my_python"), build_import("nonexistent")];
        let import_refs = imports.iter().collect();

        let (valid_imports, errors, _builder) =
            validate_imports(&model_path, builder, import_refs, &file_loader);

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
        let model_path = get_model_path();
        let builder = get_empty_builder();
        let imports = vec![
            build_import("my_python1"),
            build_import("my_python2"),
            build_import("my_python3"),
        ];
        let import_refs = imports.iter().collect();

        let (valid_imports, errors, _builder) =
            validate_imports(&model_path, builder, import_refs, &file_loader);

        assert_eq!(valid_imports.len(), 3);
        assert!(valid_imports.contains(&PythonPath::new(PathBuf::from("my_python1"))));
        assert!(valid_imports.contains(&PythonPath::new(PathBuf::from("my_python2"))));
        assert!(valid_imports.contains(&PythonPath::new(PathBuf::from("my_python3"))));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_imports_all_invalid() {
        let file_loader = TestPythonValidator::validate_none();
        let model_path = get_model_path();
        let builder = get_empty_builder();
        let imports = vec![build_import("nonexistent1"), build_import("nonexistent2")];
        let import_refs = imports.iter().collect();

        let (valid_imports, errors, _builder) =
            validate_imports(&model_path, builder, import_refs, &file_loader);

        assert!(valid_imports.is_empty());
        assert_eq!(errors.len(), 2);
        assert!(errors.contains_key(&PythonPath::new(PathBuf::from("nonexistent1"))));
        assert!(errors.contains_key(&PythonPath::new(PathBuf::from("nonexistent2"))));
    }

    #[test]
    fn test_validate_imports_builder_error_tracking() {
        let file_loader = TestPythonValidator::validate_none();
        let model_path = get_model_path();
        let builder = get_empty_builder();
        let imports = vec![build_import("nonexistent")];
        let import_refs = imports.iter().collect();

        let (_valid_imports, _errors, builder) =
            validate_imports(&model_path, builder, import_refs, &file_loader);

        assert!(
            builder
                .get_imports_with_errors()
                .contains(&PythonPath::new(PathBuf::from("nonexistent")))
        );
    }

    #[test]
    fn test_validate_imports_path_conversion() {
        let file_loader = TestPythonValidator::validate_some(vec!["subdir/my_python.py".into()]);
        let model_path = ModelPath::new(PathBuf::from("subdir/test_model"));
        let builder = get_empty_builder();
        let imports = vec![build_import("my_python")];
        let import_refs = imports.iter().collect();

        let (valid_imports, _errors, _builder) =
            validate_imports(&model_path, builder, import_refs, &file_loader);

        // The import should be converted to a Python path relative to the model
        assert_eq!(valid_imports.len(), 1);
        assert!(valid_imports.contains(&PythonPath::new(PathBuf::from("subdir/my_python"))));
    }
}
