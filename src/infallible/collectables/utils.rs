use alloc::vec::Vec;
use orx_pinned_vec::PinnedVec;
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
