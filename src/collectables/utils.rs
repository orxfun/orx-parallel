use crate::results::ValIdx;
use alloc::vec;
use alloc::vec::Vec;
use orx_pinned_vec::PinnedVec;
use orx_priority_queue::{BinaryHeap, PriorityQueue};
use orx_split_vec::{Growth, SplitVec};

// ordered

/// `results` contain `n` vectors collected by `n` threads.
/// Each vector contains (value, index) pairs represented by the `ValIdx` struct.
/// Each vector is ordered within itself; i.e., the indices are in non-decreasing order.
///
/// This method takes all elements from all the result vectors and pushes them into the `dst`
/// vector such that all elements in the destinations are in non-decreasing order of indices.
/// However, indices are not pushed to the destination, they are only used for ordering.
///
/// Ties can only happen within a result vector from a particular thread.
/// In this case, ties are broken in an order consistent with the ordering within the thread
/// result vector.
pub fn merge_ord_into<T, P>(results: Vec<Vec<ValIdx<T>>>, dst: P) -> P
where
    P: PinnedVec<T>,
    T: Send,
{
    merge_ord_into1(results, dst)
}

/// `results` contain `n` vectors collected by `n` threads.
/// Each vector contains (value, index) pairs represented by the `ValIdx` struct.
/// Each vector is ordered within itself; i.e., the indices are in non-decreasing order.
///
/// This method takes all elements from all the result vectors and pushes them into the `dst`
/// vector such that all elements in the destinations are in non-decreasing order of indices.
/// However, indices are not pushed to the destination, they are only used for ordering.
///
/// Ties can only happen within a result vector from a particular thread.
/// In this case, ties are broken in an order consistent with the ordering within the thread
/// result vector.
pub fn merge_ord_into1<T, P>(mut results: Vec<Vec<ValIdx<T>>>, mut dst: P) -> P
where
    P: PinnedVec<T>,
{
    if results.len() == 1 {
        let results = results.into_iter().next().expect("results.len()==1");
        for v in results {
            dst.push(v.val);
        }
        return dst;
    }

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
        dst.push(unsafe { ptr.add(idx).read().val });
    }

    for vec in results.iter_mut() {
        // SAFETY: this prevents to drop the elements which are already moved to pinned_vec
        // allocation within vec.capacity() will still be reclaimed; however, as uninitialized memory
        unsafe { vec.set_len(0) };
    }

    dst
}

// arbitrary

pub fn merge_arb_into_first_vec<T>(results: Vec<Vec<T>>) -> Vec<T> {
    let total_len: usize = results.iter().map(|x| x.len()).sum();
    let mut results = results.into_iter();
    match results.next() {
        None => Default::default(),
        Some(mut result) => {
            let additional = total_len - result.len();
            result.reserve(additional);
            for vec in results {
                result.extend(vec);
            }
            result
        }
    }
}

pub fn merge_arb_into_vec<T>(results: Vec<Vec<T>>, mut dst: Vec<T>) -> Vec<T> {
    let total_len: usize = results.iter().map(|x| x.len()).sum();
    dst.reserve(total_len);
    for vec in results {
        dst.extend(vec);
    }
    dst
}

pub fn merge_arb_into_split_vec<T, G: Growth>(
    results: Vec<Vec<T>>,
    mut dst: SplitVec<T, G>,
) -> SplitVec<T, G> {
    for vec in results {
        dst.extend(vec);
    }
    dst
}

#[cfg(feature = "experimental")]
mod variants {
    use super::*;

    /// Alternative ordered merge (v2): linear scan over thread heads.
    ///
    /// This avoids heap maintenance and can be faster when the number of source vectors is small.
    pub fn merge_ord_into2<T, P>(mut results: Vec<Vec<ValIdx<T>>>, mut dst: P) -> P
    where
        P: PinnedVec<T>,
    {
        if results.len() == 1 {
            let results = results.into_iter().next().expect("results.len()==1");
            for v in results {
                dst.push(v.val);
            }
            return dst;
        }

        let mut indices = vec![0; results.len()];

        loop {
            let mut chosen_v = None;
            let mut chosen_idx = usize::MAX;

            for (v, vec) in results.iter().enumerate() {
                if let Some(x) = vec.get(indices[v]) {
                    if x.idx < chosen_idx {
                        chosen_idx = x.idx;
                        chosen_v = Some(v);
                    }
                }
            }

            let Some(v) = chosen_v else {
                break;
            };

            let idx = indices[v];
            indices[v] += 1;

            let ptr = results[v].as_ptr();
            dst.push(unsafe { ptr.add(idx).read().val });
        }

        for vec in results.iter_mut() {
            // SAFETY: this prevents drop of moved-out elements while reclaiming allocation.
            unsafe { vec.set_len(0) };
        }

        dst
    }

    /// Alternative ordered merge (v3): flatten + stable sort by index.
    ///
    /// This can be beneficial when branch-heavy k-way selection becomes expensive and
    /// cache locality of a single contiguous buffer dominates.
    pub fn merge_ord_into3<T, P>(results: Vec<Vec<ValIdx<T>>>, mut dst: P) -> P
    where
        P: PinnedVec<T>,
    {
        let total_len: usize = results.iter().map(|x| x.len()).sum();
        let mut all = Vec::with_capacity(total_len);

        for vec in results {
            all.extend(vec);
        }

        // Stable sort keeps relative order among equal indices.
        all.sort_by_key(|x| x.idx);

        for x in all {
            dst.push(x.val);
        }

        dst
    }

    fn merge_two_sorted_validx<T>(left: Vec<ValIdx<T>>, right: Vec<ValIdx<T>>) -> Vec<ValIdx<T>> {
        let mut left = left.into_iter();
        let mut right = right.into_iter();

        let mut a = left.next();
        let mut b = right.next();

        let mut out = Vec::new();

        loop {
            match (a, b) {
                (Some(x), Some(y)) => {
                    if x.idx <= y.idx {
                        out.push(x);
                        a = left.next();
                        b = Some(y);
                    } else {
                        out.push(y);
                        a = Some(x);
                        b = right.next();
                    }
                }
                (Some(x), None) => {
                    out.push(x);
                    out.extend(left);
                    break;
                }
                (None, Some(y)) => {
                    out.push(y);
                    out.extend(right);
                    break;
                }
                (None, None) => break,
            }
        }

        out
    }

    /// Alternative ordered merge (v4): iterative pairwise merge tree.
    ///
    /// This remains O(total_len * log(num_vectors)) but may improve locality compared to
    /// heap-based selection, especially for larger source vectors.
    pub fn merge_ord_into4<T, P>(mut results: Vec<Vec<ValIdx<T>>>, mut dst: P) -> P
    where
        P: PinnedVec<T>,
    {
        if results.is_empty() {
            return dst;
        }

        while results.len() > 1 {
            let mut next_round = Vec::with_capacity((results.len() + 1) / 2);
            let mut it = results.into_iter();

            while let Some(left) = it.next() {
                match it.next() {
                    Some(right) => next_round.push(merge_two_sorted_validx(left, right)),
                    None => next_round.push(left),
                }
            }

            results = next_round;
        }

        for x in results.into_iter().next().expect("non-empty") {
            dst.push(x.val);
        }

        dst
    }

    /// Alternative ordered merge (v5): parallel pairwise merge tree (std-only).
    ///
    /// Uses scoped threads to merge pairs in parallel at each round.
    #[cfg(feature = "std")]
    pub fn merge_ord_into5<T, P>(mut results: Vec<Vec<ValIdx<T>>>, mut dst: P) -> P
    where
        T: Send,
        P: PinnedVec<T>,
    {
        if results.is_empty() {
            return dst;
        }

        while results.len() > 1 {
            let round_len = results.len();
            let mut it = results.into_iter();
            let mut next_round = Vec::with_capacity((round_len + 1) / 2);

            std::thread::scope(|scope| {
                let mut handles = Vec::new();

                while let Some(left) = it.next() {
                    match it.next() {
                        Some(right) => {
                            handles.push(scope.spawn(move || merge_two_sorted_validx(left, right)));
                        }
                        None => next_round.push(left),
                    }
                }

                for h in handles {
                    match h.join() {
                        Ok(v) => next_round.push(v),
                        Err(_) => panic!("merge_ord_into5 worker thread panicked"),
                    }
                }
            });

            results = next_round;
        }

        for x in results.into_iter().next().expect("non-empty") {
            dst.push(x.val);
        }

        dst
    }
}
