use std::{io::Error as IoError, path::Path};

use oneil_shared::error::AsOneilError;

pub struct FileError<'a> {
    path: &'a Path,
    error: IoError,
}

impl<'a> FileError<'a> {
    pub const fn new(path: &'a Path, error: IoError) -> Self {
        Self { path, error }
    }
}

impl AsOneilError for FileError<'_> {
    fn message(&self) -> String {
        format!("couldn't read `{}` - {}", self.path.display(), self.error)
    }
}
