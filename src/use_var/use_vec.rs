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
/// use orx_parallel::*;
///
/// let n = 10_000usize;
/// let mut partial_sums = UseVec::new(|_| 0usize);
///
/// (0..n)
///     .into_par()
///     .use_vec(&mut partial_sums)
///     .for_each(|thread_sum, x| *thread_sum += x);
///
/// let total: usize = partial_sums.into_vec().into_iter().sum();
/// assert_eq!(total, (n - 1) * n / 2);
/// ```
pub struct UseVec<T: Send, F: Fn(usize) -> T + Sync> {
    init: F,
    cache: ConcurrentOrderedBag<T>,
}

impl<T: Send, F: Fn(usize) -> T + Sync> UseVec<T, F> {
    /// Creates a `UseVec` with per-thread initialization logic.
    ///
    /// The `init` function receives the worker `thread_idx` and is invoked on
    /// first access to create that thread's local value.
    ///
    /// # Example
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut calls = UseVec::new(|_| 0usize);
    ///
    /// let values: Vec<_> = (0..128usize)
    ///     .into_par()
    ///     .use_vec(&mut calls)
    ///     .map(|count, x| {
    ///         *count += 1;
    ///         x * 2
    ///     })
    ///     .collect();
    ///
    /// assert_eq!(values.len(), 128);
    /// assert_eq!(calls.into_vec().into_iter().sum::<usize>(), 128);
    /// ```
    pub fn new(init: F) -> Self {
        let cache = ConcurrentOrderedBag::new();
        Self { init, cache }
    }

    /// Consumes the `UseVec` and returns the per-thread values as a `Vec`.
    ///
    /// This is typically used after a parallel computation to aggregate thread-local state.
    ///
    /// # Example
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut partial_sums = UseVec::new(|_| 0usize);
    ///
    /// (0..100usize)
    ///     .into_par()
    ///     .num_threads(4)
    ///     .use_vec(&mut partial_sums)
    ///     .for_each(|thread_sum, x| *thread_sum += x);
    ///
    /// let partials = partial_sums.into_vec();
    /// assert_eq!(partials.into_iter().sum::<usize>(), 4950);
    /// ```
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
