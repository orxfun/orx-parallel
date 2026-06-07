use super::r#use::Use;
use alloc::vec::Vec;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;

/// Owned worker-local mutable state.
///
/// `UseVec` stores one value per worker thread and lets parallel operations
/// mutate those values independently. It is typically used with
/// [`Par::use_vec`](crate::Par::use_vec).
///
/// # Examples
///
/// ```
/// use orx_parallel::{Use, UseVec};
///
/// let use_vec = UseVec::new(|thread_idx| thread_idx + 10);
///
/// assert_eq!(*use_vec.init_get(0), 10);
/// assert_eq!(*use_vec.init_get(1), 11);
///
/// assert_eq!(use_vec.into_vec(), vec![10, 11]);
/// ```
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

    fn init_get(&self, thread_idx: usize) -> &mut Self::Item {
        let use_var = (self.init)(thread_idx);
        unsafe { self.cache.set_value(thread_idx, use_var) };

        // SAFETY: it is safe to access to the index as it is
        // pushed / initialized just above. Further, `get` will
        // be called exactly once by the corresponding thread,
        // and hence, there will be no race condition.
        unsafe { &mut *self.cache.ptr_mut(thread_idx) }
    }

    #[inline]
    fn get(&mut self, thread_idx: usize) -> &mut Self::Item {
        assert!(self.cache.len() > thread_idx);
        unsafe { &mut *self.cache.ptr_mut(thread_idx) }
    }

    fn max_threads(&self) -> Option<usize> {
        None
    }
}

impl<T: Send, F: Fn(usize) -> T + Sync> Use for &mut UseVec<T, F> {
    type Item = T;

    fn init_get(&self, thread_idx: usize) -> &mut Self::Item {
        let use_var = (self.init)(thread_idx);
        unsafe { self.cache.set_value(thread_idx, use_var) };

        // SAFETY: it is safe to access to the index as it is
        // pushed / initialized just above. Further, `get` will
        // be called exactly once by the corresponding thread,
        // and hence, there will be no race condition.
        unsafe { &mut *self.cache.ptr_mut(thread_idx) }
    }

    #[inline]
    fn get(&mut self, thread_idx: usize) -> &mut Self::Item {
        assert!(self.cache.len() > thread_idx);
        unsafe { &mut *self.cache.ptr_mut(thread_idx) }
    }

    fn max_threads(&self) -> Option<usize> {
        None
    }
}
