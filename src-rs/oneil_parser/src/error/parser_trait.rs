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
    /// Converts errors to a new type using the provided conversion function.
    ///
    /// This method takes a function that converts errors (`nom::Err::Error`) into a new error type,
    /// while preserving unrecoverable errors (`nom::Err::Failure`) by using `From` conversion.
    ///
    /// # Arguments
    ///
    /// * `convert_error` - A function that converts errors to the new error type
    ///
    /// # Type Parameters
    ///
    /// * `E2` - The target error type that implements `ParseError<I>` and can be created `From<E>`
    ///
    /// # Example
    ///
    /// ```ignore
    /// use nom::Parser;
    /// let parser = identifier.convert_error_to(|e| MyError::from_nom_error(e));
    /// ```
    fn convert_error_to<E2>(
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

    /// Maps recoverable errors to unrecoverable errors using the provided conversion function.
    ///
    /// This method takes a function that converts recoverable errors (`nom::Err::Error`) into a new error type,
    /// while preserving unrecoverable errors (`nom::Err::Failure`) by using `From` conversion.
    ///
    /// # Arguments
    ///
    /// * `convert_error` - A function that converts recoverable errors to the new error type
    ///
    /// # Type Parameters
    ///
    /// * `E2` - The target error type that implements `ParseError<I>` and can be created `From<E>`
    ///
    /// # Example
    ///
    /// ```ignore
    /// use nom::Parser;
    /// let parser = identifier.or_fail_with(ParserError::expect_identifier);
    /// ```
    fn or_fail_with<E2>(
        mut self,
        convert_error: impl Fn(E) -> E2,
    ) -> impl Parser<I, Output = O, Error = E2>
    where
        Self: Sized,
        E2: nom::error::ParseError<I> + From<E>,
    {
        move |input| {
            self.parse(input).map_err(|e| match e {
                nom::Err::Error(e) => nom::Err::Failure(convert_error(e)),
                nom::Err::Failure(e) => nom::Err::Failure(e.into()),
                nom::Err::Incomplete(e) => nom::Err::Incomplete(e),
            })
        }
    }

    /// Converts errors to a new type that implements `From<E>`.
    ///
    /// This is a convenience method that uses `Into` for both recoverable and
    /// unrecoverable errors.
    fn convert_errors<E2>(mut self) -> impl Parser<I, Output = O, Error = E2>
    where
        Self: Sized,
        E2: nom::error::ParseError<I> + From<E>,
    {
        move |input| {
            self.parse(input).map_err(|e| match e {
                nom::Err::Error(e) => nom::Err::Error(e.into()),
                nom::Err::Failure(e) => nom::Err::Failure(e.into()),
                nom::Err::Incomplete(e) => nom::Err::Incomplete(e),
            })
        }
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
