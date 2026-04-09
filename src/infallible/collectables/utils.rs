use crate::results::ValIdx;
use alloc::vec;
use alloc::vec::Vec;
use orx_pinned_vec::PinnedVec;
use orx_priority_queue::{BinaryHeap, PriorityQueue};
use orx_split_vec::{Growth, SplitVec};

pub fn extend_vec_from_split<T, G>(
    mut initial_vec: Vec<T>,
    collected_split_vec: SplitVec<T, G>,
) -> Vec<T>
where
    G: Growth,
{
    match initial_vec.len() {
        0 => collected_split_vec.to_vec(),
        _ => {
            initial_vec.reserve(collected_split_vec.len());
            initial_vec.extend(collected_split_vec);
            initial_vec
        }
    }
}

pub fn split_vec_reserve<T, G: Growth>(split_vec: &mut SplitVec<T, G>, iter_len: Option<usize>) {
    match iter_len {
        None => {
            let capacity_bound = split_vec.capacity_bound();
            split_vec.reserve_maximum_concurrent_capacity(capacity_bound)
        }
        Some(len) => split_vec.reserve_maximum_concurrent_capacity(split_vec.len() + len),
    };
}

pub fn merge_collected_into<T>(mut results: Vec<Vec<ValIdx<T>>>, mut dst: Vec<T>) -> Vec<T> {
    if results.len() == 1 {
        let results = results.into_iter().next().expect("results.len()==1");
        dst.extend(results.into_iter().map(|x| x.val));
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
