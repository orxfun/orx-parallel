use alloc::vec::Vec;

/// Value and index of an element.
pub struct ValIdx<T> {
    pub val: T,
    pub idx: usize,
}

impl<T> ValIdx<T> {
    #[inline(always)]
    pub fn new(val: T, idx: usize) -> Self {
        Self { val, idx }
    }

    /// Find and returns the value & index pair from the `results` which has the minimum index.
    pub fn first(results: Vec<Option<Self>>) -> Option<Self> {
        let mut min_idx = usize::MAX;
        let mut value = None;

        for x in results {
            if let Some(y) = x {
                if y.idx < min_idx {
                    min_idx = y.idx;
                    value = Some(y);
                }
            }
        }

        value
    }

    pub fn first_res<E>(results: Vec<Result<Option<Self>, E>>) -> Result<Option<Self>, E> {
        let mut min_idx = usize::MAX;
        let mut value = None;

        for x in results {
            match x {
                Ok(Some(y)) => {
                    if y.idx < min_idx {
                        min_idx = y.idx;
                        value = Some(y);
                    }
                }
                Ok(None) => {}
                Err(e) => return Err(e),
            }
        }

        Ok(value)
    }

    /// Returns either of the following:
    ///
    /// * Some(Some(found)): no failure, found an element
    /// * Some(None): no failure but no element to find
    /// * None: a failure (None) is observed
    pub fn first_opt(results: Vec<Option<Option<Self>>>) -> Option<Option<Self>> {
        let mut min_idx = usize::MAX;
        let mut value = None;

        for x in results {
            match x {
                Some(Some(y)) => {
                    if y.idx < min_idx {
                        min_idx = y.idx;
                        value = Some(y);
                    }
                }
                Some(None) => {}
                None => return None,
            }
        }

        Some(value)
    }
}
