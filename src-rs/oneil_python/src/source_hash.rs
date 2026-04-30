use std::path::Path;

use indexmap::IndexMap;
use xxhash_rust::xxh3::Xxh3Default;

use crate::LoadPythonImportError;

pub fn calculate_source_hash(source_paths: Vec<&Path>) -> Result<u64, LoadPythonImportError> {
    // sort the source paths to ensure consistent hashing
    let mut source_paths = source_paths;
    source_paths.sort_unstable(); // `Path` implements `Ord` correctly, so `unstable` is okay

    // hash the source files

    let mut builder = Xxh3Default::new();
    let mut file_errors = IndexMap::new();

    for source_path in source_paths {
        let source = match std::fs::read_to_string(source_path) {
            Ok(source) => source,
            Err(e) => {
                file_errors.insert(source_path.to_path_buf(), e);
                continue;
            }
        };

        builder.update(source.as_bytes());
    }

    // if there are any file errors, return an error
    if !file_errors.is_empty() {
        return Err(LoadPythonImportError::CouldNotCalculateSourceHash {
            file_errors: Box::new(file_errors),
        });
    }

    // digest the source files
    let hash = builder.digest();

    Ok(hash)
}
