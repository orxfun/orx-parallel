use crate::runner::results::ValIdx;
use alloc::vec::Vec;

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
    #[inline]
    pub fn from_maybe(maybe: Option<T>, idx: usize) -> Self {
        match maybe {
            Some(val) => Self::Success(ValIdx::new(val, idx)),
            None => Self::Fail(idx),
        }
    }

    /// Find and returns the value & index pair from the `results` which has the minimum index.
    /// If at least one of the elements is a failure, this method will also return the failure
    /// with the minimum index.
    pub fn first_of(results: Vec<Option<Self>>) -> Option<Self> {
        let mut success = (usize::MAX, None);
        let mut failure = (usize::MAX, None);

        for x in results {
            if let Some(y) = x {
                match y {
                    Self::Success(z) => {
                        if z.idx < success.0 {
                            success = (z.idx, Some(z))
                        }
                    }
                    Self::Fail(idx) => {
                        if idx < failure.0 {
                            failure = (idx, Some(idx))
                        }
                    }
                }
            }
        }

        match (failure.1, success.1) {
            (Some(e), _) => Some(Self::Fail(e)),
            (None, Some(s)) => Some(Self::Success(s)),
            (None, None) => None,
        }
    }
}
