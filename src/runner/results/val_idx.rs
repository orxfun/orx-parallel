use alloc::vec::Vec;

pub struct ValIdx<T> {
    pub val: T,
    pub idx: usize,
}

impl<T> ValIdx<T> {
    #[inline(always)]
    pub fn new(val: T, idx: usize) -> Self {
        Self { val, idx }
    }

    pub fn find_next(results: Vec<Option<ValIdx<T>>>) -> Option<ValIdx<T>> {
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
}
