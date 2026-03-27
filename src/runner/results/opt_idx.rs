use crate::runner::results::ValIdx;

/// Value & index result of a fallible operation.
pub enum OptIdx<T> {
    /// Successful result without observing any failures; i.e., None's in the iterator.
    Success(ValIdx<T>),
    /// At least one failure (None) is observed at the stored index.
    Fail(usize),
}

impl<T> OptIdx<T> {
    /// Creates the fallible value & index result from the observed element `maybe` at
    /// the index `idx`:
    ///
    /// * returns Success variant with the value and index if `maybe.is_some()`,
    /// * returns Fail with the cached index otherwise.
    pub fn from_maybe(maybe: Option<T>, idx: usize) -> Self {
        match maybe {
            Some(val) => Self::Success(ValIdx::new(val, idx)),
            None => Self::Fail(idx),
        }
    }
}
