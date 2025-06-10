use nom::Parser;

pub trait ErrorHandlingParser<I, O, E>: Parser<I, Output = O, Error = E>
where
    E: nom::error::ParseError<I>,
{
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

    fn errors_into<E2>(self) -> impl Parser<I, Output = O, Error = E2>
    where
        Self: Sized,
        E2: nom::error::ParseError<I> + From<E>,
    {
        self.map_error_and_failure(|e| e.into(), |e| e.into())
    }
}

impl<'a, I, O, E, P> ErrorHandlingParser<I, O, E> for P
where
    P: Parser<I, Output = O, Error = E>,
    E: nom::error::ParseError<I>,
{
}
