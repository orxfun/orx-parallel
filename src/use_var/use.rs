/// Worker-local mutable state used by parallel iterators.
///
/// A `Use` value provides one mutable slot per participating worker thread.
/// Parallel combinators initialize the slot for a thread on first access and
/// then reuse it for subsequent accesses from the same thread.
///
/// [`UseVec`](crate::UseVec) is the owned implementation of this trait in this crate.
///
/// # Examples
///
/// ```
/// use orx_parallel::{Use, UseVec};
///
/// fn bump_first_thread<U: Use<Item = usize>>(use_var: &mut U) -> usize {
///     *use_var.init_get(0) += 1;
///     *use_var.get(0)
/// }
///
/// let mut use_vec = UseVec::new(|_| 0usize);
/// assert_eq!(bump_first_thread(&mut use_vec), 1);
/// assert_eq!(use_vec.into_vec(), vec![1]);
/// ```
pub trait Use: Sync {
    /// Type of the worker-local mutable value stored for each thread.
    type Item;

    /// Returns the mutable worker-local value for `thread_idx`, creating it if needed.
    #[allow(clippy::mut_from_ref)]
    fn init_get(&self, thread_idx: usize) -> &mut Self::Item;

    /// Returns the already-initialized mutable worker-local value for `thread_idx`.
    ///
    /// # Panics
    ///
    /// Panics if the corresponding slot has not been initialized by a previous
    /// call to [`init_get`](Self::init_get).
    fn get(&mut self, thread_idx: usize) -> &mut Self::Item;

    /// Returns an upper bound on the number of worker threads that may use this value.
    ///
    /// `None` means the implementation does not impose a fixed upper bound.
    fn max_threads(&self) -> Option<usize>;
}
