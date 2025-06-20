use nom::{Parser, error::ParseError};

/// A trait for handling parser errors in a consistent way.
///
/// This trait extends nom's `Parser` trait with additional error handling capabilities,
/// providing methods to:
///
/// - Convert between different error types
/// - Handle both recoverable (Error) and unrecoverable (Failure) errors
/// - Map errors while preserving the error type hierarchy
///
/// # Type Parameters
///
/// * `I` - The input type (usually `Span`)
/// * `O` - The output type
/// * `E` - The error type
pub trait ErrorHandlingParser<I, O, E>: Parser<I, Output = O, Error = E>
where
    E: ParseError<I>,
{
    /// Maps recoverable errors while preserving unrecoverable errors.
    ///
    /// This is useful when you want to convert only the recoverable errors
    /// to a different type, leaving the unrecoverable errors as-is. This uses `Into`
    /// to convert the errors.
    fn map_error<E2>(
        mut self,
        convert_error: impl Fn(E) -> E2,
    ) -> impl Parser<I, Output = O, Error = E2>
    where
        Self: Sized,
        E2: nom::error::ParseError<I> + From<E>,
    {
        move |input| {
            self.parse(input).map_err(|e| match e {
                nom::Err::Error(e) => nom::Err::Error(convert_error(e)),
                nom::Err::Failure(e) => nom::Err::Failure(e.into()),
                nom::Err::Incomplete(e) => nom::Err::Incomplete(e),
            })
        }
    }

    /// Maps unrecoverable errors while preserving recoverable errors.
    ///
    /// This is useful when you want to convert only the unrecoverable errors
    /// to a different type, leaving the recoverable errors as-is. This uses `Into`
    /// to convert the errors.
    fn map_failure<E2>(
        mut self,
        convert_failure: impl Fn(E) -> E2,
    ) -> impl Parser<I, Output = O, Error = E2>
    where
        Self: Sized,
        E2: nom::error::ParseError<I> + From<E>,
    {
        move |input| {
            self.parse(input).map_err(|e| match e {
                nom::Err::Error(e) => nom::Err::Error(e.into()),
                nom::Err::Failure(e) => nom::Err::Failure(convert_failure(e)),
                nom::Err::Incomplete(e) => nom::Err::Incomplete(e),
            })
        }
    }

    /// Maps both recoverable and unrecoverable errors independently.
    ///
    /// This is the most flexible error mapping function, allowing different
    /// conversions for recoverable and unrecoverable errors.
    fn map_error_and_failure<E2>(
        mut self,
        convert_error: impl Fn(E) -> E2,
        convert_failure: impl Fn(E) -> E2,
    ) -> impl Parser<I, Output = O, Error = E2>
    where
        Self: Sized,
        E2: nom::error::ParseError<I>,
    {
        move |input| {
            self.parse(input).map_err(|e| match e {
                nom::Err::Error(e) => nom::Err::Error(convert_error(e)),
                nom::Err::Failure(e) => nom::Err::Failure(convert_failure(e)),
                nom::Err::Incomplete(e) => nom::Err::Incomplete(e),
            })
        }
    }

    /// Converts errors to a new type that implements `From<E>`.
    ///
    /// This is a convenience method that uses `Into` for both recoverable and
    /// unrecoverable errors.
    fn convert_errors<E2>(self) -> impl Parser<I, Output = O, Error = E2>
    where
        Self: Sized,
        E2: nom::error::ParseError<I> + From<E>,
    {
        self.map_error_and_failure(|e| e.into(), |e| e.into())
    }
}

/// Implements the ErrorHandlingParser trait for any type that implements Parser.
///
/// This blanket implementation allows any parser to use the error handling
/// methods provided by ErrorHandlingParser.
impl<'a, I, O, E, P> ErrorHandlingParser<I, O, E> for P
where
    P: Parser<I, Output = O, Error = E>,
    E: ParseError<I>,
{
}
