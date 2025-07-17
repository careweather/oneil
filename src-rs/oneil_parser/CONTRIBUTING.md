# Contributing to Oneil Parser

Thank you for your interest in contributing to the Oneil parser! This document provides guidelines and standards for contributing to the parser implementation.

## Overview

The Oneil parser is a Rust implementation of a parser for the Oneil programming language. It uses a recursive descent parsing approach with the [nom] parsing library.

## Development Setup

1. **Prerequisites**
   - Rust 1.70+ (check with `rustc --version`)
   - Cargo (comes with Rust)

2. **Clone and Setup**
   ```bash
   git clone https://github.com/careweather/oneil
   cd src-rs/oneil_parser
   cargo check
   ```

3. **Run Tests**
   ```bash
   # Run all tests
   cargo test
   
   # Run documentation tests
   cargo test --doc
   
   # Run tests with output
   cargo test -- --nocapture
   ```

## Code Style and Standards

### Rust Conventions

- Follow the [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/style/naming/README.html)
- Use `rustfmt` to format code: `cargo fmt`
- Use `clippy` for linting: `cargo clippy`

### Naming Conventions

- **Modules**: Use snake_case (e.g., `token`, `error`, `model`)
- **Functions**: Use snake_case (e.g., `parse_model`, `parse_expression`)
- **Types**: Use PascalCase (e.g., `ParserError`, `ModelNode`)
- **Constants**: Use SCREAMING_SNAKE_CASE (e.g., `MAX_TOKEN_LENGTH`)

### Error Handling

- Use `Result<T, E>` for all fallible operations
- Define clear custom error types
- Provide detailed error messages with location information
- Use the `?` operator for error propagation

## Documentation Standards

### Module Documentation

Every module should have comprehensive documentation at the top:

```rust
//! Parser for [specific language construct] in the Oneil language.
//!
//! This module provides parsing functionality for [description of what it parses].
//! It handles [specific aspects] and integrates with [other modules].
//!
//! # Key Features
//!
//! - [Feature 1]: [Description]
//! - [Feature 2]: [Description]
//!
//! # Examples
//!
//! ```rust
//! use oneil_parser::[module]::[function];
//!
//! let result = [function]("example input", None)?;
//! ```
```

### Function Documentation

Every public function must have complete documentation:

```rust
/// Parses a [specific construct] from source code.
///
/// This function handles [detailed description of what it does],
/// including [specific cases or edge cases].
///
/// # Arguments
///
/// * `input` - The source code to parse
/// * `config` - Optional parser configuration
///
/// # Returns
///
/// Returns `Ok(T)` on successful parsing, or `Err(E)` with detailed error information.
///
/// # Examples
///
/// ```rust
/// use oneil_parser::[module]::[function];
///
/// // Basic usage
/// let result = [function]("valid input", None)?;
///
/// // With configuration
/// let config = Config::new();
/// let result = [function]("input", Some(config))?;
/// ```
///
/// # Errors
///
/// This function will return an error if:
/// - [Error condition 1]
/// - [Error condition 2]
pub fn function_name(input: &str, config: Option<Config>) -> Result<T, E> {
    // Implementation
}
```

### Type Documentation

All public types should be well-documented:

```rust
/// Represents a [description of what this type represents].
///
/// This type is used for [purpose/context] and provides [key functionality].
/// It implements [traits] and can be used for [use cases].
///
/// # Examples
///
/// ```rust
/// use oneil_parser::[module]::[TypeName];
///
/// let instance = TypeName::new(/* parameters */);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct TypeName {
    /// [Description of this field]
    pub field_name: FieldType,
}
```

### Documentation Testing

Ensure all doc tests pass: `cargo test --doc`

## Parser Architecture

### Module Organization

The parser is organized into several modules:

- **`lib.rs`**: Main public API and entry points
- **`config.rs`**: Parser configuration options
- **`error/`**: Comprehensive error handling system
- **`token/`**: Low-level tokenization and lexical analysis
- **`model.rs`**: Complete model parsing
- **`declaration.rs`**: Import, from, and use declarations
- **`expression.rs`**: Mathematical expressions
- **`parameter.rs`**: Parameter definitions
- **`test.rs`**: Test definitions
- **`unit.rs`**: Unit expressions
- **`note.rs`**: Documentation comments

### Error Handling Strategy

The parser uses a two-level error handling approach:

1. **Token-level errors** (`TokenError`): For low-level tokenizing issues
2. **Parser-level errors** (`ParserError`): For higher-level parsing issues

Error types include:
- **Location**: Precise character offset where the error occurred
- **Context**: Information about what was expected vs. what was found
- **Recovery**: Partial parsing results when possible

## Testing Guidelines

### Unit Tests

- Test each parsing function individually
- Include both success and failure cases
- Test edge cases and error conditions
- Use descriptive test names

```rust
#[test]
fn test_parse_valid_expression() {
    let result = parse_expression("2 + 3", None);
    assert!(result.is_ok());
}

#[test]
fn test_parse_invalid_expression() {
    let result = parse_expression("2 + + 3", None);
    assert!(result.is_err());
}
```

### Documentation Tests

- Ensure all public API examples work
- Test error handling examples
- Verify configuration examples

## Adding New Language Features

When adding new language features:

1. **Update Grammar**: Add the feature to `/docs/specs/grammar.ebnf`
2. **Create Parser**: Implement parsing logic in appropriate module
3. **Add Tests**: Include comprehensive test coverage
4. **Update Documentation**: Document the new feature thoroughly
5. **Update Error Handling**: Add appropriate error types and messages

## Pull Request Process

1. **Fork** the repository
2. **Create** a feature branch: `git checkout -b feature/description`
3. **Make** your changes following the guidelines above
4. **Test** your changes thoroughly
5. **Document** any new functionality
6. **Submit** a pull request with a clear description

### Pull Request Checklist

- [ ] Code follows Rust conventions and project style
- [ ] All tests pass (`cargo test`)
- [ ] Documentation tests pass (`cargo test --doc`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] New functionality is documented
- [ ] Error handling is appropriate
- [ ] Grammar is updated if needed

## Getting Help

- Check existing documentation and examples
- Review similar implementations in the codebase
- Ask questions in issues or discussions
- Consult the [nom documentation](https://docs.rs/nom/) for parsing patterns

## License

By contributing to this project, you agree that your contributions will be licensed under the same terms as the project. 