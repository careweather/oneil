use oneil_shared::error::OneilError;

pub struct PartialResultWithErrors<T> {
    pub result: T,
    pub errors: Vec<OneilError>,
}
