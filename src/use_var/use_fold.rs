use crate::use_var::{Use, UseVec, pair_ptr::PairPtr};
use alloc::vec::Vec;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;

pub struct UseFold<U, T, F>
where
    U: Use,
    T: Send,
    F: Fn(usize) -> T + Sync,
{
    u: U,
    v: UseVec<T, F>,
    cache: ConcurrentOrderedBag<PairPtr<U::Item, T>>,
}

impl<U, T, F> UseFold<U, T, F>
where
    U: Use,
    T: Send,
    F: Fn(usize) -> T + Sync,
{
    pub fn new(u: U, init: F) -> Self {
        let v = UseVec::new(init);
        let cache = ConcurrentOrderedBag::new();
        Self { u, v, cache }
    }

    pub fn into_vec(self) -> Vec<T> {
        let _ = unsafe { self.cache.into_inner().unwrap_only_if_counts_match() };
        self.v.into_vec()
    }
}

impl<U, T, F> Use for &mut UseFold<U, T, F>
where
    U: Use,
    T: Send,
    F: Fn(usize) -> T + Sync,
{
    type Item = PairPtr<U::Item, T>;

    fn init_get(&self, thread_idx: usize) -> &mut Self::Item {
        let u = self.u.init_get(thread_idx) as *mut U::Item;
        let v = self.v.init_get(thread_idx) as *mut T;
        let pair_ptr = PairPtr::new(u, v);
        unsafe { self.cache.set_value(thread_idx, pair_ptr) };

        // SAFETY: it is safe to access to the index as it is
        // pushed / initialized just above. Further, `get` will
        // be called exactly once by the corresponding thread,
        // and hence, there will be no race condition.
        unsafe { &mut *self.cache.ptr_mut(thread_idx) }
    }

    fn get(&mut self, thread_idx: usize) -> &mut Self::Item {
        assert!(self.cache.len() > thread_idx);
        unsafe { &mut *self.cache.ptr_mut(thread_idx) }
    }

    fn max_threads(&self) -> Option<usize> {
        match (self.u.max_threads(), self.v.max_threads()) {
            (Some(u), Some(v)) => Some(u.min(v)),
            (Some(u), None) => Some(u),
            (None, Some(v)) => Some(v),
            (None, None) => None,
        }
    }
}
