use alloc::vec::Vec;

/// Value result of a fallible operation, where observing None at any element is considered failure.
pub enum Opt<T> {
    /// Successful result without observing any failures; i.e., None's in the iterator.
    Success(T),
    /// At least one failure (None) is observed.
    Fail,
}

impl<T> Opt<T> {
    #[inline]
    pub fn from_maybe(maybe: Option<T>) -> Self {
        match maybe {
            Some(val) => Self::Success(val),
            None => Self::Fail,
        }
    }

    pub fn any_of(results: Vec<Option<Self>>) -> Option<Self> {
        let mut result = None;

        for x in results {
            if let Some(y) = x {
                match (&result, &y) {
                    (None, Self::Success(_)) => result = Some(y),
                    (_, Self::Fail) => return Some(Self::Fail),
                    _ => {}
                }
            }
        }

        result
    }
}
