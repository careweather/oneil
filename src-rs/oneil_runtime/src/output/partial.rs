use oneil_shared::error::OneilError;

pub struct PartialResultWithErrors<T, E> {
    pub result: T,
    pub errors: E,
}
