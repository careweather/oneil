use std::collections::HashMap;

use oneil_ast as ast;
use oneil_ir as ir;

use crate::{
    FileLoader, error::resolution::ImportResolutionError, util::builder::ModelCollectionBuilder,
};

type ValidatedImports = HashMap<ir::PythonPath, IrSpan>;
type ImportErrors = HashMap<ir::PythonPath, ImportResolutionError>;

/// Validates a list of Python import declarations for a given model.
pub fn resolve_python_imports<F>(
    model_path: &ir::ModelPath,
    builder: ModelCollectionBuilder<F::ParseError, F::PythonError>,
    imports: Vec<&ast::ImportNode>,
    file_loader: &F,
) -> (
    ValidatedImports,
    ImportErrors,
    ModelCollectionBuilder<F::ParseError, F::PythonError>,
)
where
    F: FileLoader,
{
    // TODO: change this into a for loop
    imports.into_iter().fold(
        (HashMap::new(), HashMap::new(), builder),
        |(mut python_imports, mut import_resolution_errors, mut builder), import| {
            let python_path = model_path.get_sibling_path(import.path());
            let python_path = ir::PythonPath::new(python_path);
            let python_path_span = get_span_from_ast_span(import.path().node_span());

            // check for duplicate imports
            let original_import_span = python_imports.get(&python_path);
            if let Some(original_import_span) = original_import_span {
                import_resolution_errors.insert(
                    python_path.clone(),
                    ImportResolutionError::duplicate_import(
                        *original_import_span,
                        python_path_span,
                        python_path,
                    ),
                );

                return (python_imports, import_resolution_errors, builder);
            }

            let result = file_loader.validate_python_import(&python_path);
            match result {
                Ok(()) => {
                    python_imports.insert(python_path, python_path_span);
                    (python_imports, import_resolution_errors, builder)
                }
                Err(error) => {
                    builder.add_import_error(python_path.clone(), error);
                    import_resolution_errors.insert(
                        python_path.clone(),
                        ImportResolutionError::failed_validation(python_path_span, python_path),
                    );
                    (python_imports, import_resolution_errors, builder)
                }
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::test::{
        TestPythonValidator,
        construct::{self, test_ast},
    };

    #[test]
    fn validate_imports_empty_list() {
        // set up the context
        let file_loader = TestPythonValidator::validate_all();
        let model_path = ir::ModelPath::new("test_model");
        let builder = construct::empty_model_collection_builder();

        // set up the imports
        let imports = vec![];

        // validate the imports
        let (valid_imports, errors, _builder) =
            resolve_python_imports(&model_path, builder, imports, &file_loader);

        // check the imports
        assert!(valid_imports.is_empty());

        // check the errors
        assert!(errors.is_empty());
    }

    #[test]
    fn validate_imports_single_valid_import() {
        // set up the context
        let file_loader = TestPythonValidator::validate_all();
        let model_path = ir::ModelPath::new("test_model");
        let builder = construct::empty_model_collection_builder();

        // set up the imports
        let imports = [test_ast::ImportPythonNodeBuilder::build("my_python")];
        let import_refs = imports.iter().collect();

        // validate the imports
        let (valid_imports, errors, _builder) =
            resolve_python_imports(&model_path, builder, import_refs, &file_loader);

        // check the imports
        assert_eq!(valid_imports.len(), 1);

        let valid_path = ir::PythonPath::new(PathBuf::from("my_python"));
        assert!(valid_imports.contains_key(&valid_path));

        // check the errors
        assert!(errors.is_empty());
    }

    #[test]
    fn validate_imports_single_invalid_import() {
        // set up the context
        let file_loader = TestPythonValidator::validate_none();
        let model_path = ir::ModelPath::new("test_model");
        let builder = construct::empty_model_collection_builder();

        // set up the imports
        let imports = [test_ast::ImportPythonNodeBuilder::build("nonexistent")];
        let import_refs = imports.iter().collect();

        // validate the imports
        let (valid_imports, errors, _builder) =
            resolve_python_imports(&model_path, builder, import_refs, &file_loader);

        // check the imports
        assert!(valid_imports.is_empty());

        // check the errors
        assert_eq!(errors.len(), 1);

        let error_path = ir::PythonPath::new(PathBuf::from("nonexistent"));
        let error = errors.get(&error_path).expect("error should be present");

        let ImportResolutionError::FailedValidation {
            ident_span: _,
            python_path: error_path_actual,
        } = error
        else {
            panic!("error should be a failed validation error");
        };

        assert_eq!(error_path_actual, &error_path);
    }

    #[test]
    fn validate_imports_mixed_valid_and_invalid() {
        // set up the context
        let file_loader = TestPythonValidator::validate_some(["my_python.py"]);
        let model_path = ir::ModelPath::new("test_model");
        let builder = construct::empty_model_collection_builder();

        // set up the imports
        let imports = [
            test_ast::ImportPythonNodeBuilder::build("my_python"),
            test_ast::ImportPythonNodeBuilder::build("nonexistent"),
        ];
        let import_refs = imports.iter().collect();

        // validate the imports
        let (valid_imports, errors, _builder) =
            resolve_python_imports(&model_path, builder, import_refs, &file_loader);

        // check the imports
        assert_eq!(valid_imports.len(), 1);

        let valid_path = ir::PythonPath::new(PathBuf::from("my_python"));
        assert!(valid_imports.contains_key(&valid_path));

        // check the errors
        assert_eq!(errors.len(), 1);

        let error_path = ir::PythonPath::new(PathBuf::from("nonexistent"));
        let error = errors.get(&error_path).expect("error should be present");

        let ImportResolutionError::FailedValidation {
            ident_span: _,
            python_path: error_path_actual,
        } = error
        else {
            panic!("error should be a failed validation error");
        };

        assert_eq!(error_path_actual, &error_path);
    }

    #[test]
    fn validate_imports_multiple_valid_imports() {
        // set up the context
        let file_loader = TestPythonValidator::validate_all();
        let model_path = ir::ModelPath::new("test_model");
        let builder = construct::empty_model_collection_builder();

        // set up the imports
        let imports = [
            test_ast::ImportPythonNodeBuilder::build("my_python1"),
            test_ast::ImportPythonNodeBuilder::build("my_python2"),
            test_ast::ImportPythonNodeBuilder::build("my_python3"),
        ];
        let import_refs = imports.iter().collect();

        // validate the imports
        let (valid_imports, errors, _builder) =
            resolve_python_imports(&model_path, builder, import_refs, &file_loader);

        // check the imports
        assert_eq!(valid_imports.len(), 3);
        let valid_path1 = ir::PythonPath::new(PathBuf::from("my_python1"));
        assert!(valid_imports.contains_key(&valid_path1));

        let valid_path2 = ir::PythonPath::new(PathBuf::from("my_python2"));
        assert!(valid_imports.contains_key(&valid_path2));

        let valid_path3 = ir::PythonPath::new(PathBuf::from("my_python3"));
        assert!(valid_imports.contains_key(&valid_path3));

        // check the errors
        assert!(errors.is_empty());
    }

    #[test]
    fn validate_imports_all_invalid() {
        // set up the context
        let file_loader = TestPythonValidator::validate_none();
        let model_path = ir::ModelPath::new("test_model");
        let builder = construct::empty_model_collection_builder();

        // set up the imports
        let imports = [
            test_ast::ImportPythonNodeBuilder::build("nonexistent1"),
            test_ast::ImportPythonNodeBuilder::build("nonexistent2"),
        ];
        let import_refs = imports.iter().collect();

        // validate the imports
        let (valid_imports, errors, _builder) =
            resolve_python_imports(&model_path, builder, import_refs, &file_loader);

        // check the imports
        assert!(valid_imports.is_empty());

        // check the errors
        assert_eq!(errors.len(), 2);

        let error_path = ir::PythonPath::new(PathBuf::from("nonexistent1"));
        let error = errors.get(&error_path).expect("error should be present");

        let ImportResolutionError::FailedValidation {
            ident_span: _,
            python_path: error_path_actual,
        } = error
        else {
            panic!("error should be a failed validation error");
        };

        assert_eq!(error_path_actual, &error_path);

        let error_path = ir::PythonPath::new(PathBuf::from("nonexistent2"));
        let error = errors.get(&error_path).expect("error should be present");

        let ImportResolutionError::FailedValidation {
            ident_span: _,
            python_path: error_path_actual,
        } = error
        else {
            panic!("error should be a failed validation error");
        };

        assert_eq!(error_path_actual, &error_path);
    }

    #[test]
    fn validate_imports_builder_error_tracking() {
        // set up the context
        let file_loader = TestPythonValidator::validate_none();
        let model_path = ir::ModelPath::new("test_model");
        let builder = construct::empty_model_collection_builder();

        // set up the imports
        let imports = [test_ast::ImportPythonNodeBuilder::build("nonexistent")];
        let import_refs = imports.iter().collect();

        // validate the imports
        let (_valid_imports, _errors, builder) =
            resolve_python_imports(&model_path, builder, import_refs, &file_loader);

        // check the builder
        assert!(
            builder
                .get_imports_with_errors()
                .contains(&ir::PythonPath::new(PathBuf::from("nonexistent")))
        );
    }

    #[test]
    fn validate_imports_path_conversion() {
        // set up the context
        let file_loader = TestPythonValidator::validate_some(["subdir/my_python.py"]);
        let model_path = ir::ModelPath::new(PathBuf::from("subdir/test_model"));
        let builder = construct::empty_model_collection_builder();

        // set up the imports
        let imports = [test_ast::ImportPythonNodeBuilder::build("my_python")];
        let import_refs = imports.iter().collect();

        // validate the imports
        let (valid_imports, errors, _builder) =
            resolve_python_imports(&model_path, builder, import_refs, &file_loader);

        // check the imports
        assert_eq!(valid_imports.len(), 1);

        let valid_path = ir::PythonPath::new(PathBuf::from("subdir/my_python"));
        assert!(valid_imports.contains_key(&valid_path));

        // check the errors
        assert!(errors.is_empty());
    }

    #[test]
    fn validate_imports_duplicate_imports() {
        // set up the context
        let file_loader = TestPythonValidator::validate_all();
        let model_path = ir::ModelPath::new("test_model");
        let builder = construct::empty_model_collection_builder();

        // set up the imports with different spans to simulate different positions in the file
        let imports = [
            // first import
            test_ast::ImportPythonNodeBuilder::build("my_python"),
            // duplicate import
            test_ast::ImportPythonNodeBuilder::build("my_python"),
        ];
        let import_refs = imports.iter().collect();

        // validate the imports
        let (valid_imports, errors, _builder) =
            resolve_python_imports(&model_path, builder, import_refs, &file_loader);

        // check the imports - only the first one should be valid
        assert_eq!(valid_imports.len(), 1);

        let valid_path = ir::PythonPath::new(PathBuf::from("my_python"));
        assert!(valid_imports.contains_key(&valid_path));

        // check the errors - should have one duplicate import error
        assert_eq!(errors.len(), 1);

        let error_path = ir::PythonPath::new(PathBuf::from("my_python"));
        let duplicate_error = errors
            .get(&error_path)
            .expect("duplicate error should be present");

        let ImportResolutionError::DuplicateImport {
            python_path: duplicate_error_path,
            ..
        } = duplicate_error
        else {
            panic!("duplicate error should be a duplicate import error");
        };

        assert_eq!(duplicate_error_path, &error_path);
    }

    #[test]
    fn validate_imports_multiple_duplicate_imports() {
        // set up the context
        let file_loader = TestPythonValidator::validate_all();
        let model_path = ir::ModelPath::new("test_model");
        let builder = construct::empty_model_collection_builder();

        // set up the imports with multiple duplicates
        let imports = [
            // first import
            test_ast::ImportPythonNodeBuilder::build("my_python"),
            // different import
            test_ast::ImportPythonNodeBuilder::build("other_python"),
            // duplicate of first
            test_ast::ImportPythonNodeBuilder::build("my_python"),
            // duplicate of second
            test_ast::ImportPythonNodeBuilder::build("other_python"),
        ];
        let import_refs = imports.iter().collect();

        // validate the imports
        let (valid_imports, errors, _builder) =
            resolve_python_imports(&model_path, builder, import_refs, &file_loader);

        // check the imports - only the first occurrence of each should be valid
        assert_eq!(valid_imports.len(), 2);

        let valid_path1 = ir::PythonPath::new(PathBuf::from("my_python"));
        assert!(valid_imports.contains_key(&valid_path1));

        let valid_path2 = ir::PythonPath::new(PathBuf::from("other_python"));
        assert!(valid_imports.contains_key(&valid_path2));

        // check the errors - should have two duplicate import errors
        assert_eq!(errors.len(), 2);

        let duplicate_error1 = errors
            .get(&valid_path1)
            .expect("duplicate error should be present");

        let ImportResolutionError::DuplicateImport {
            python_path: duplicate_error_path1,
            ..
        } = duplicate_error1
        else {
            panic!("duplicate error should be a duplicate import error");
        };

        assert_eq!(duplicate_error_path1, &valid_path1);

        let duplicate_error2 = errors
            .get(&valid_path2)
            .expect("duplicate error should be present");

        let ImportResolutionError::DuplicateImport {
            python_path: duplicate_error_path2,
            ..
        } = duplicate_error2
        else {
            panic!("duplicate error should be a duplicate import error");
        };

        assert_eq!(duplicate_error_path2, &valid_path2);
    }

    #[test]
    fn validate_imports_duplicate_imports_with_invalid_imports() {
        // set up the context
        let file_loader = TestPythonValidator::validate_some(["my_python.py"]);
        let model_path = ir::ModelPath::new("test_model");
        let builder = construct::empty_model_collection_builder();

        // set up the imports with duplicates and invalid imports
        let imports = [
            // valid import
            test_ast::ImportPythonNodeBuilder::build("my_python"),
            // invalid import
            test_ast::ImportPythonNodeBuilder::build("nonexistent"),
            // duplicate of first
            test_ast::ImportPythonNodeBuilder::build("my_python"),
            // another invalid import
            test_ast::ImportPythonNodeBuilder::build("another_nonexistent"),
        ];
        let import_refs = imports.iter().collect();

        // validate the imports
        let (valid_imports, errors, _builder) =
            resolve_python_imports(&model_path, builder, import_refs, &file_loader);

        // check the imports - only the first valid import should be present
        assert_eq!(valid_imports.len(), 1);

        let valid_path = ir::PythonPath::new(PathBuf::from("my_python"));
        assert!(valid_imports.contains_key(&valid_path));

        // check the errors - should have 3 errors: 1 duplicate + 2 invalid imports
        assert_eq!(errors.len(), 3);

        // Check duplicate import error
        let duplicate_error = errors
            .get(&valid_path)
            .expect("duplicate error should be present");

        let ImportResolutionError::DuplicateImport {
            python_path: duplicate_error_path,
            ..
        } = duplicate_error
        else {
            panic!("duplicate error should be a duplicate import error");
        };

        assert_eq!(duplicate_error_path, &valid_path);

        // Check invalid import errors
        let invalid_path1 = ir::PythonPath::new(PathBuf::from("nonexistent"));
        let invalid_error1 = errors
            .get(&invalid_path1)
            .expect("invalid error should be present");

        let ImportResolutionError::FailedValidation {
            python_path: invalid_path1_actual,
            ..
        } = invalid_error1
        else {
            panic!("invalid error should be a failed validation error");
        };

        assert_eq!(invalid_path1_actual, &invalid_path1);

        let invalid_path2 = ir::PythonPath::new(PathBuf::from("another_nonexistent"));
        let invalid_error2 = errors
            .get(&invalid_path2)
            .expect("invalid error should be present");

        let ImportResolutionError::FailedValidation {
            python_path: invalid_path2_actual,
            ..
        } = invalid_error2
        else {
            panic!("invalid error should be a failed validation error");
        };

        assert_eq!(invalid_path2_actual, &invalid_path2);
    }
}
