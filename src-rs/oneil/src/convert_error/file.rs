use std::{io::Error as IoError, path::Path};

use crate::convert_error::Error;

pub fn convert(path: &Path, error: &IoError) -> Error {
    let message = format!("couldn't read `{}` - {}", path.display(), error);
    Error::new(path.to_path_buf(), message)
}
