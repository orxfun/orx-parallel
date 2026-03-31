use alloc::vec::Vec;
use orx_pinned_vec::PinnedVec;
use orx_split_vec::{GrowthWithConstantTimeAccess, SplitVec};

pub fn extend_vec_from_split<T, G>(
    mut initial_vec: Vec<T>,
    collected_split_vec: SplitVec<T, G>,
) -> Vec<T>
where
    G: GrowthWithConstantTimeAccess,
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

pub fn split_vec_reserve<T, G: GrowthWithConstantTimeAccess>(
    split_vec: &mut SplitVec<T, G>,
    is_sequential: bool,
    iter_len: Option<usize>,
) {
    let len_to_extend = match (is_sequential, iter_len) {
        (true, _) => None, // not required to concurrent reserve when seq
        (false, x) => x,
    };

    match len_to_extend {
        None => {
            let capacity_bound = split_vec.capacity_bound();
            split_vec.reserve_maximum_concurrent_capacity(capacity_bound)
        }
        Some(len) => split_vec.reserve_maximum_concurrent_capacity(split_vec.len() + len),
    };
}
