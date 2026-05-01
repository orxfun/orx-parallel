use crate::results::ValsAndIdx;
use alloc::vec;
use alloc::vec::Vec;
use orx_pinned_vec::PinnedVec;
use orx_priority_queue::{BinaryHeap, PriorityQueue};
use orx_split_vec::{Growth, PseudoDefault, SplitVec};

#[derive(Clone)]
struct VecPos {
    v: usize,
    beg: usize,
    len: usize,
}

impl VecPos {
    #[inline(always)]
    fn new(v: usize, beg: usize, len: usize) -> Self {
        Self { v, beg, len }
    }
}

// vec

// TODO: performance: this method can be parallelized by pairwise merging
pub fn merge_ord_into_vec<T>(mut results: Vec<ValsAndIdx<T>>, dst: &mut Vec<T>) {
    let collected_len: usize = results.iter().map(|x| x.values.len()).sum();
    dst.reserve(collected_len);
    let initial_len = dst.len();
    let total_len = initial_len + collected_len;

    let mut queue = BinaryHeap::with_capacity(results.len());
    let mut pos_indices = vec![0; results.len()];

    for (v, vec) in results.iter().enumerate() {
        if let Some(pos) = vec.positions.get(0) {
            queue.push(VecPos::new(v, 0, pos.len), pos.idx);
        }
    }
    let mut curr_v = queue.pop_node();
    let mut ptr_dst = unsafe { dst.as_mut_ptr().add(initial_len) };

    while let Some(VecPos { v, beg, len }) = curr_v {
        let ptr_src = unsafe { results[v].values.as_ptr().add(beg) };
        unsafe { ptr_dst.copy_from_nonoverlapping(ptr_src, len) };

        pos_indices[v] += 1;
        curr_v = match results[v].positions.get(pos_indices[v]) {
            Some(pos) => {
                let beg = beg + len;
                Some(queue.push_then_pop(VecPos::new(v, beg, pos.len), pos.idx).0)
            }
            None => queue.pop_node(),
        };

        ptr_dst = unsafe { ptr_dst.add(len) };
    }

    for vec in results.iter_mut() {
        // SAFETY: this prevents to drop the elements which are already moved to pinned_vec
        // allocation within vec.capacity() will still be reclaimed; however, as uninitialized memory
        unsafe { vec.values.set_len(0) };
    }

    unsafe { dst.set_len(total_len) };
}

// splitvec

// TODO: performance: this method can be parallelized by pairwise merging
pub fn merge_ord_into_split_vec<T, G: Growth + PseudoDefault>(
    mut results: Vec<ValsAndIdx<T>>,
    dst: &mut SplitVec<T, G>,
) {
    let mut queue = BinaryHeap::with_capacity(results.len());
    let mut pos_indices = vec![0; results.len()];

    for (v, vec) in results.iter().enumerate() {
        if let Some(pos) = vec.positions.get(0) {
            queue.push(VecPos::new(v, 0, pos.len), pos.idx);
        }
    }
    let mut curr_v = queue.pop_node();

    while let Some(VecPos { v, beg, len }) = curr_v {
        let ptr_src = unsafe { results[v].values.as_ptr().add(beg) };
        unsafe { dst.extend_from_nonoverlapping(ptr_src, len) };

        pos_indices[v] += 1;
        curr_v = match results[v].positions.get(pos_indices[v]) {
            Some(pos) => {
                let beg = beg + len;
                Some(queue.push_then_pop(VecPos::new(v, beg, pos.len), pos.idx).0)
            }
            None => queue.pop_node(),
        };
    }

    for vec in results.iter_mut() {
        // SAFETY: this prevents to drop the elements which are already moved to pinned_vec
        // allocation within vec.capacity() will still be reclaimed; however, as uninitialized memory
        unsafe { vec.values.set_len(0) };
    }
}
