//! Error types for runtime output operations.

use std::path::PathBuf;

pub struct TreeError {
    model_path: PathBuf,
    parameter_name: String,
}
