use crate::collectables_old::Collectable;
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

pub fn merge_arb<T, S, D>(results: Vec<S>, dst: &mut D)
where
    S: Collectable<T>,
    D: Collectable<T>,
{
    let total_len: usize = results.iter().map(|x| x.col_len()).sum();
    dst.col_reserve(total_len);
    for vec in results {
        dst.col_extend(vec);
    }
}
