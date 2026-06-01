use crate::use_var::Use;
use alloc::vec::Vec;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;

pub struct UseCached<U: Use> {
    u: U,
    cache: ConcurrentOrderedBag<U::Item>,
}

impl<U: Use> UseCached<U> {
    pub fn new(u: U) -> Self {
        let cache = ConcurrentOrderedBag::new();
        Self { u, cache }
    }

    pub fn into_vec(self) -> Vec<U::Item> {
        let vec = unsafe { self.cache.into_inner().unwrap_only_if_counts_match() };
        vec.into_iter().collect()
    }
}
