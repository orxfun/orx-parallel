use crate::results::ValIdx;
use alloc::vec;
use alloc::vec::Vec;
use orx_pinned_vec::PinnedVec;
use orx_priority_queue::{BinaryHeap, PriorityQueue};
use orx_split_vec::{Growth, SplitVec};

pub fn merge_ord_into<T, P>(results: Vec<Vec<ValIdx<T>>>, dst: P) -> P
where
    P: PinnedVec<T>,
    T: Send,
{
    // merge_ord_into1(results, dst)
    todo!()
}
