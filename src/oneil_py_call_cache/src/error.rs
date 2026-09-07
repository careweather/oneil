//! Errors for cache file I/O.

use std::fmt;

/// Failure when writing a cache JSON file.
#[derive(Debug)]
pub enum WriteCacheError {
    /// I/O error creating or writing the file.
    Io(std::io::Error),
    /// JSON serialization error.
    Serde(serde_json::Error),
}

impl fmt::Display for WriteCacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "failed to write cache file: {e}"),
            Self::Serde(e) => write!(f, "failed to serialize cache to JSON: {e}"),
        }
    }
}

impl std::error::Error for WriteCacheError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Serde(e) => Some(e),
        }
    }
}

/// Failure when reading a cache JSON file.
#[derive(Debug)]
pub enum ReadCacheError {
    /// I/O error opening or reading the file.
    Io(std::io::Error),
    /// JSON deserialization error.
    Serde(serde_json::Error),
}

impl fmt::Display for ReadCacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "failed to read cache file: {e}"),
            Self::Serde(e) => write!(f, "failed to deserialize cache from JSON: {e}"),
        }
    }
}

impl std::error::Error for ReadCacheError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Serde(e) => Some(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error as _;
    use std::io;

    use super::{ReadCacheError, WriteCacheError};

    #[test]
    fn write_cache_io_error_display_includes_io_message() {
        let error = WriteCacheError::Io(io::Error::other("disk full"));

        let message = error.to_string();

        assert_eq!(message, "failed to write cache file: disk full");
    }

    #[test]
    fn write_cache_io_error_source_matches_wrapped_error() {
        let io_error = io::Error::other("disk full");
        let expected_kind = io_error.kind();
        let expected_message = io_error.to_string();
        let error = WriteCacheError::Io(io_error);

        let source = error.source().expect("I/O error should have a source");
        let source = source
            .downcast_ref::<io::Error>()
            .expect("source should be an I/O error");

        assert_eq!(source.kind(), expected_kind);
        assert_eq!(source.to_string(), expected_message);
    }

    #[test]
    fn read_cache_io_error_display_includes_io_message() {
        let error = ReadCacheError::Io(io::Error::other("not found"));

        let message = error.to_string();

        assert_eq!(message, "failed to read cache file: not found");
    }

    #[test]
    fn read_cache_io_error_source_matches_wrapped_error() {
        let io_error = io::Error::other("not found");
        let expected_kind = io_error.kind();
        let expected_message = io_error.to_string();
        let error = ReadCacheError::Io(io_error);

        let source = error.source().expect("I/O error should have a source");
        let source = source
            .downcast_ref::<io::Error>()
            .expect("source should be an I/O error");

        assert_eq!(source.kind(), expected_kind);
        assert_eq!(source.to_string(), expected_message);
    }

    #[test]
    fn read_cache_serde_error_display_includes_context() {
        let serde_error = serde_json::from_str::<serde_json::Value>("{")
            .expect_err("invalid json should fail to parse");
        let error = ReadCacheError::Serde(serde_error);

        let message = error.to_string();

        assert!(message.starts_with("failed to deserialize cache from JSON:"));
    }

    #[test]
    fn read_cache_serde_error_source_matches_wrapped_error() {
        let serde_error = serde_json::from_str::<serde_json::Value>("{")
            .expect_err("invalid json should fail to parse");
        let expected_message = serde_error.to_string();
        let expected_line = serde_error.line();
        let expected_column = serde_error.column();
        let error = ReadCacheError::Serde(serde_error);

        let source = error.source().expect("serde error should have a source");
        let source = source
            .downcast_ref::<serde_json::Error>()
            .expect("source should be a serde error");

        assert_eq!(source.to_string(), expected_message);
        assert_eq!(source.line(), expected_line);
        assert_eq!(source.column(), expected_column);
    }
}
