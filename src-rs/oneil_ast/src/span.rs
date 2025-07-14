#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    start: usize,
    end: usize,
    whitespace_end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, whitespace_end: usize) -> Self {
        Self {
            start,
            end,
            whitespace_end,
        }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn whitespace_end(&self) -> usize {
        self.whitespace_end
    }

    pub fn calc_span<T, U>(start_span: &T, end_span: &U) -> Self
    where
        T: SpanLike,
        U: SpanLike,
    {
        Self::new(
            start_span.get_start(),
            end_span.get_end(),
            end_span.get_whitespace_end(),
        )
    }
}

pub trait SpanLike {
    fn get_start(&self) -> usize;
    fn get_end(&self) -> usize;
    fn get_whitespace_end(&self) -> usize;
}

impl SpanLike for Span {
    fn get_start(&self) -> usize {
        self.start
    }

    fn get_end(&self) -> usize {
        self.end
    }

    fn get_whitespace_end(&self) -> usize {
        self.whitespace_end
    }
}

impl<T, U> From<(&T, &U)> for Span
where
    T: SpanLike,
    U: SpanLike,
{
    fn from((start_span, end_span): (&T, &U)) -> Self {
        Self::calc_span(start_span, end_span)
    }
}

impl<T> From<&T> for Span
where
    T: SpanLike,
{
    fn from(span: &T) -> Self {
        Self::new(span.get_start(), span.get_end(), span.get_whitespace_end())
    }
}
