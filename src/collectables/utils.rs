use crate::results::ValIdx;
use alloc::vec;
use alloc::vec::Vec;
use orx_pinned_vec::PinnedVec;
use orx_priority_queue::{BinaryHeap, PriorityQueue};
use orx_split_vec::{Growth, SplitVec};

// ordered

pub fn merge_ord_into<T, P>(mut results: Vec<Vec<ValIdx<T>>>, mut dst: P) -> P
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
