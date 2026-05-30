use alloc::vec::Vec;
use orx_concurrent_bag::ConcurrentBag;
use orx_pinned_vec::PinnedVec;

pub struct UseVec<T, F: Fn(usize) -> T> {
    init: F,
    cache: ConcurrentBag<(usize, T)>,
}

impl<T, F: Fn(usize) -> T> UseVec<T, F> {
    pub fn new(init: F) -> Self {
        let cache = ConcurrentBag::new();
        Self { init, cache }
    }

    pub fn into_vec(self) -> Vec<T> {
        let mut vec = self.cache.into_inner();
        vec.sort_by_key(|(th_idx, _)| *th_idx);
        vec.into_iter().map(|(_, value)| value).collect()
    }
}
