use crate::infallible_use::Using;
use alloc::vec::Vec;
use orx_concurrent_bag::ConcurrentBag;
use orx_pinned_vec::PinnedVec;

pub struct UseVec<T: Send, F: Fn(usize) -> T + Sync> {
    init: F,
    cache: ConcurrentBag<(usize, T)>,
}

impl<T: Send, F: Fn(usize) -> T + Sync> UseVec<T, F> {
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

impl<T: Send, F: Fn(usize) -> T + Sync> Using for UseVec<T, F> {
    type Item = T;

    type ItemBorrow<'i>
        = &'i mut T
    where
        Self: 'i;

    fn create(&self, thread_idx: usize) -> Self::Item {
        todo!()
    }

    fn get(&self, thread_idx: usize) -> Self::ItemBorrow<'_> {
        let use_var = (self.init)(thread_idx);
        let idx = self.cache.push((thread_idx, use_var));
        // SAFETY: it is safe to access to the index as it is
        // pushed / initialized just above. Further, `get` will
        // be called exactly once by the corresponding thread,
        // and hence, there will be no race condition.
        let a = unsafe { &mut *self.cache.ptr_mut(idx) };
        &mut a.1
    }
}
