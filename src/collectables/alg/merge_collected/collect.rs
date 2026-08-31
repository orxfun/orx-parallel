use crate::collectables::Collectable;
use alloc::vec::Vec;

pub struct Collect;

impl Collect {
    pub fn merge_results_arb<T, S, D>(results: Vec<S>, dst: &mut D)
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
}
