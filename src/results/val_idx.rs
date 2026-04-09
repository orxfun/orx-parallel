use alloc::vec;
use alloc::vec::Vec;
use orx_pinned_vec::{IntoConcurrentPinnedVec, PinnedVec};
use orx_priority_queue::{BinaryHeap, PriorityQueue};

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

    pub fn collect_into<P>(mut results: Vec<Vec<Self>>, pinned_vec: &mut P)
    where
        P: PinnedVec<T>,
    {
        let mut queue = BinaryHeap::with_capacity(results.len());
        let mut indices = vec![0; results.len()];

        for (v, vec) in results.iter().enumerate() {
            if let Some(x) = vec.get(indices[v]) {
                queue.push(v, x.idx);
            }
        }
        let mut curr_v = queue.pop_node();

        while let Some(v) = curr_v {
            let idx = indices[v];
            indices[v] += 1;

            curr_v = match results[v].get(indices[v]) {
                Some(x) => Some(queue.push_then_pop(v, x.idx).0),
                None => queue.pop_node(),
            };

            let ptr = results[v].as_ptr();
            pinned_vec.push(unsafe { ptr.add(idx).read().val });
        }

        for vec in results.iter_mut() {
            // SAFETY: this prevents to drop the elements which are already moved to pinned_vec
            // allocation within vec.capacity() will still be reclaimed; however, as uninitialized memory
            unsafe { vec.set_len(0) };
        }
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
