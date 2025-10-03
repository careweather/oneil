# Oneil Error

A unified error handling system for the Oneil programming language.

This crate enables components to use their own error types while also defining a unified interface with which to work.

The main feature of this library is the `AsOneilError` trait found in [`traits.rs`](src/traits.rs). Errors should implement this trait in order to be compatible with Oneil CLI error printing.

## Example

```rust
use oneil_error::{OneilError, AsOneilError, Context};
use std::path::PathBuf;

// Define an error type that implements AsOneilError
struct MyError {
    message: String,
    offset: usize,
}

impl AsOneilError for MyError {
    fn message(&self) -> String {
        self.message.clone()
    }

    fn error_location(&self, source: &str) -> Option<oneil_error::ErrorLocation> {
        if self.offset < source.len() {
            Some(oneil_error::ErrorLocation::from_source_and_offset(source, self.offset))
        } else {
            None
        }
    }

    fn context(&self) -> Vec<Context> {
        vec![Context::Help("Try checking your syntax".to_string())]
    }
}

// Convert to OneilError
let my_error = MyError {
    message: "Unexpected token".to_string(),
    offset: 10,
};

let source = "My X: x = $";
let path = PathBuf::from("example.on");
let oneil_error = OneilError::from_error_with_source(&my_error, path, source);
```
