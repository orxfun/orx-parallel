use alloc::vec::Vec;
use orx_split_vec::{Growth, SplitVec};

pub fn merge_arb_into_vec<T>(results: Vec<Vec<T>>, dst: &mut Vec<T>) {
    let total_len: usize = results.iter().map(|x| x.len()).sum();
    dst.reserve(total_len);
    for vec in results {
        dst.extend(vec);
    }
}

pub fn merge_arb_into_split_vec<T, G: Growth>(results: Vec<Vec<T>>, dst: &mut SplitVec<T, G>) {
    for vec in results {
        dst.extend(vec);
    }
}
