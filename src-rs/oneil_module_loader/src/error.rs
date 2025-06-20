#[derive(Debug, Clone, PartialEq)]
pub enum ModuleLoaderError<E> {
    ParseError(E),
}

impl<E> From<E> for ModuleLoaderError<E> {
    fn from(e: E) -> Self {
        ModuleLoaderError::ParseError(e)
    }
}
