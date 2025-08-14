# Error Handling Model

The Oneil parser uses a sophisticated error handling system built on top of nom's error handling, with two main types of errors: "not found" errors and "incomplete" errors. The system also includes a trait-based approach for consistent error handling across parser components.

## Error Types Overview

### "Not found" errors

**Summary:**
- Related to `nom::Err::Error`
- Example: parsing `foo` as a number
- Represented by `nom::Err::Error` since another parser may succeed on this input

"Not found" errors occur when a token is not found at the expected location. For example, if we are trying to parse the input `foo` and we expect a number, this is a "not found" error.

These errors are handled by the `token` function for tokens, which will return an associated error kind. The error is represented by `nom::Err::Error` because it's possible that another parser may succeed on this input. In the case of `foo`, the identifier parser would succeed.

### Incomplete errors

**Summary:**
- Related to `nom::Err::Failure`
- Example: parsing `"foo` (unclosed string)
- Indicated by the use of `or_fail_with()` which converts `nom::Err::Error` to `nom::Err::Failure`

Incomplete errors occur when a token is clearly started, but the parser is unable to complete the token. For example, if we are trying to parse the input `"foo` and we expect a string, we start to parse the string, but we reach the end of the input without finding a closing quote. This is an incomplete error.

Incomplete errors are indicated by the use of `or_fail_with()`, which converts a `nom::Err::Error` to a `nom::Err::Failure`. Incomplete errors should be converted to the correct error kind *when it is "cut"*, since other error handlers should only be handling `nom::Err::Error`, not `nom::Err::Failure`.

## Error Handling Trait

The parser uses the `ErrorHandlingParser` trait to provide consistent error handling across all parser components. This trait extends nom's `Parser` trait with additional error handling capabilities.

### Key Methods

#### `convert_error_to<E2>(convert_error: impl Fn(E) -> E2)`

Converts errors to a new type using the provided conversion function. This method:
- Takes a function that converts errors (`nom::Err::Error`) into a new error type
- Preserves unrecoverable errors (`nom::Err::Failure`) by using `From` conversion
- Maintains the error type hierarchy

**Example:**
```rust
let parser = identifier.convert_error_to(|e| MyError::from_nom_error(e));
```

#### `or_fail_with<E2>(convert_error: impl Fn(E) -> E2)`

Maps recoverable errors to unrecoverable errors using the provided conversion function. This method:
- Takes a function that converts recoverable errors (`nom::Err::Error`) into a new error type
- Converts them to `nom::Err::Failure` to indicate incomplete parsing
- Preserves existing unrecoverable errors by using `From` conversion

**Example:**
```rust
let parser = identifier.or_fail_with(ParserError::expect_identifier);
```

#### `convert_errors<E2>()`

Converts errors to a new type that implements `From<E>`. This is a convenience method that uses `Into` for both recoverable and unrecoverable errors.

**Example:**
```rust
let parser = identifier.convert_errors::<ParserError>();
```

## Error Hierarchy

The error system has a two-level hierarchy:

### Token-level errors (`TokenError`)

Low-level parsing issues like:
- Invalid characters
- Unterminated strings
- Invalid number formats
- Missing delimiters

**Example:**
```rust
// In token/literal.rs
let (rest, _) = tag("'")
    .or_fail_with(TokenError::unclosed_string(open_quote_span))
    .parse(rest)?;
```

### Parser-level errors (`ParserError`)

Higher-level issues like:
- Invalid syntax
- Unexpected tokens
- Missing required components
- Structural parsing errors

**Example:**
```rust
// In declaration.rs
let (rest, _) = tag("import")
    .or_fail_with(ParserError::import_missing_path(&import_token))
    .parse(rest)?;
```

## Error Conversion Patterns

### Token to Parser Error Conversion

Token errors are converted to parser errors using specific conversion functions:

```rust
// Converting a token error to a parser error
ParserError::expect_note(token_error)
```

### Error Type Conversion

The system supports converting between different error types while preserving the error hierarchy:

```rust
// Converting from TokenError to ParserError
impl From<TokenError> for ParserError {
    fn from(e: TokenError) -> Self {
        Self {
            reason: ParserErrorReason::token_error(e.kind),
            error_offset: e.offset,
        }
    }
}
```

## Practical Examples

### String Parsing with Incomplete Error

```rust
pub fn string(input: Span) -> Result<Token, TokenError> {
    token(
        |input| {
            let (rest, open_quote_span) = tag("\'").parse(input)?;
            let (rest, _) = take_while(|c: char| c != '\'' && c != '\n').parse(rest)?;
            let (rest, _) = tag("'")
                .or_fail_with(TokenError::unclosed_string(open_quote_span))
                .parse(rest)?;
            Ok((rest, ()))
        },
        TokenError::expected_string,
    )
    .parse(input)
}
```

In this example:
- If the input doesn't start with a quote, it's a "not found" error (`nom::Err::Error`)
- If the input starts with a quote but doesn't end with one, `or_fail_with()` converts it to an incomplete error (`nom::Err::Failure`)

### Number Parsing with Incomplete Error

```rust
let opt_decimal = opt(|input| -> Result<_, TokenError> {
    let (rest, decimal_point_span) = tag(".").parse(input)?;
    let (rest, _) = digit1
        .or_fail_with(TokenError::invalid_decimal_part(decimal_point_span))
        .parse(rest)?;
    Ok((rest, ()))
});
```

In this example:
- If there's no decimal point, it's a "not found" error (handled by `opt`)
- If there's a decimal point but no digits after it, `or_fail_with()` converts it to an incomplete error

## Error Recovery Strategy

The parser implements a sophisticated error recovery strategy:

1. **Recoverable errors** (`nom::Err::Error`): Allow other parsers to attempt parsing the same input
2. **Unrecoverable errors** (`nom::Err::Failure`): Indicate that parsing has definitively failed and should not be retried
3. **Error conversion**: Maintains proper error context while converting between error types
4. **Partial results**: Some parsers can return partial results even when errors occur

This approach ensures that the parser provides meaningful error messages while maintaining the ability to recover from certain types of parsing failures.