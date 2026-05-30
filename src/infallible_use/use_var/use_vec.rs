use crate::infallible_use::Use;
use alloc::vec::Vec;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;

pub struct UseVec<T: Send, F: Fn(usize) -> T + Sync> {
    init: F,
    cache: ConcurrentOrderedBag<T>,
}

impl<T: Send, F: Fn(usize) -> T + Sync> UseVec<T, F> {
    pub fn new(init: F) -> Self {
        let cache = ConcurrentOrderedBag::new();
        Self { init, cache }
    }

    pub fn into_vec(self) -> Vec<T> {
        let vec = unsafe { self.cache.into_inner().unwrap_only_if_counts_match() };
        vec.into_iter().collect()
    }
}

impl<T: Send, F: Fn(usize) -> T + Sync> Use for UseVec<T, F> {
    type Item = T;

    type ItemBorrow<'i>
        = &'i mut T
    where
        Self: 'i;

    fn get(&self, thread_idx: usize) -> Self::ItemBorrow<'_> {
        let use_var = (self.init)(thread_idx);
        unsafe { self.cache.set_value(thread_idx, use_var) };

        // let idx = self.cache.push((thread_idx, use_var));
        // SAFETY: it is safe to access to the index as it is
        // pushed / initialized just above. Further, `get` will
        // be called exactly once by the corresponding thread,
        // and hence, there will be no race condition.
        unsafe { &mut *self.cache.ptr_mut(thread_idx) }
    }

    #[inline]
    fn get_mut(&mut self, thread_idx: usize) -> Self::ItemBorrow<'_> {
        assert!(self.cache.len() > 0);
        unsafe { &mut *self.cache.ptr_mut(thread_idx) }
    }
}
