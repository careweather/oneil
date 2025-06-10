use nom::Parser;

pub trait ErrorHandlingParser<I, O, E>: Parser<I, Output = O, Error = E> {
    fn map_error<E2, E3>(
        mut self,
        convert_error: impl Fn(E) -> E2,
    ) -> impl Parser<I, Output = O, Error = E3>
    where
        Self: Sized,
        E: nom::error::ParseError<I>,
        E2: nom::error::ParseError<I>,
        E3: internal::ErrorOrFailure<I, E2, E>,
    {
        move |input| {
            self.parse(input).map_err(|e| match e {
                nom::Err::Error(e) => {
                    nom::Err::Error(internal::ErrorOrFailure::error(convert_error(e)))
                }
                nom::Err::Failure(e) => nom::Err::Failure(internal::ErrorOrFailure::failure(e)),
                nom::Err::Incomplete(e) => nom::Err::Incomplete(e),
            })
        }
    }

    fn map_failure<E2, E3>(
        mut self,
        convert_failure: impl Fn(E) -> E2,
    ) -> impl Parser<I, Output = O, Error = E3>
    where
        Self: Sized,
        E: nom::error::ParseError<I>,
        E2: nom::error::ParseError<I>,
        E3: internal::ErrorOrFailure<I, E, E2>,
    {
        move |input| {
            self.parse(input).map_err(|e| match e {
                nom::Err::Error(e) => nom::Err::Error(internal::ErrorOrFailure::error(e)),
                nom::Err::Failure(e) => {
                    nom::Err::Failure(internal::ErrorOrFailure::failure(convert_failure(e)))
                }
                nom::Err::Incomplete(e) => nom::Err::Incomplete(e),
            })
        }
    }

    fn merge_error_and_failure<E1>(mut self) -> impl Parser<I, Output = O, Error = E1>
    where
        Self: Sized,
        E: internal::ErrorOrFailure<I, E1, E1>,
        E1: nom::error::ParseError<I>,
    {
        move |input| {
            self.parse(input).map_err(|e| match e {
                nom::Err::Error(e) => nom::Err::Error(internal::ErrorOrFailure::get_error(e)),
                nom::Err::Failure(e) => nom::Err::Failure(internal::ErrorOrFailure::get_failure(e)),
                nom::Err::Incomplete(e) => nom::Err::Incomplete(e),
            })
        }
    }
}

impl<'a, I, O, E, P> ErrorHandlingParser<I, O, E> for P where P: Parser<I, Output = O, Error = E> {}

pub type ErrorOrFailure<E1, E2> = internal::ErrorOrFailureImpl<E1, E2>;

mod internal {
    /// A trait used to differentiate between errors and failures and implemented by
    /// `ErrorOrFailureInternal`
    ///
    /// Technically, `nom` has the ability to differentiate between errors and failures
    /// with the `nom::Err<F, E = F>` enum, but most of the library assumes that `E` is
    /// the same as `F`, so it's difficult to use in practice.
    ///
    /// The enum must follow the invariant that `Error` is only used for `nom::Err::Error`
    /// and `Failure` is only used for `nom::Err::Failure`. This enum should not escape
    ///
    /// It's unfortunate that this hack has to be used (it's the equivalent of
    /// introducing dynamic typing), but until we find another way, this is the best
    /// we can do.
    pub trait ErrorOrFailure<I, E1, E2>: nom::error::ParseError<I> {
        fn error(error: E1) -> Self;
        fn failure(failure: E2) -> Self;
        fn get_error(self) -> E1;
        fn get_failure(self) -> E2;
    }

    pub enum ErrorOrFailureImpl<E1, E2> {
        Error(E1),
        Failure(E2),
    }

    impl<I, E1, E2> nom::error::ParseError<I> for ErrorOrFailureImpl<E1, E2> {
        fn from_error_kind(_input: I, _kind: nom::error::ErrorKind) -> Self {
            panic!("This should never be called")
        }

        fn append(_input: I, _kind: nom::error::ErrorKind, other: Self) -> Self {
            other
        }
    }

    impl<I, E1, E2> ErrorOrFailure<I, E1, E2> for ErrorOrFailureImpl<E1, E2> {
        fn error(error: E1) -> Self {
            Self::Error(error)
        }

        fn failure(failure: E2) -> Self {
            Self::Failure(failure)
        }

        fn get_error(self) -> E1 {
            match self {
                Self::Error(e) => e,
                Self::Failure(_) => panic!("`ErrorOrFailure` invariant violated"),
            }
        }

        fn get_failure(self) -> E2 {
            match self {
                Self::Error(_) => panic!("`ErrorOrFailure` invariant violated"),
                Self::Failure(e) => e,
            }
        }
    }
}
