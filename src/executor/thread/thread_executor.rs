use crate::xap::Xap;
use orx_concurrent_iter::ConcurrentIter;

/// Thread executor responsible for executing sub-tasks sequentially on a single thread.
pub trait ThreadExecutor: Sized {
    /// Type of the shared state among threads.
    type SharedState;

    /// Returns the next chunks size to pull from the input `iter` for the given
    /// current `shared_state`.
    fn next_chunk_size<I>(&self, shared_state: &Self::SharedState, iter: &I) -> usize
    where
        I: ConcurrentIter;

    /// Hook that will be called before starting to execute the chunk of the given `chunk_size`.
    fn begin_chunk(&mut self, chunk_size: usize);

    /// Hook that will be called after completing the chunk of the given `chunk_size`.
    /// The `shared_state` is also provided so that it can be updated to send information to the
    /// parallel executor and other thread executors.
    fn complete_chunk(&mut self, shared_state: &Self::SharedState, chunk_size: usize);

    /// Hook that will be called after completing the task.
    /// The `shared_state` is also provided so that it can be updated to send information to the
    /// parallel executor and other thread executors.
    fn complete_task(&mut self, shared_state: &Self::SharedState);

    // computations

    #[inline]
    fn reduce<I, X, F>(&mut self, state: &Self::SharedState, iter: &I, x: X, f: F) -> Option<X::O>
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item>,
        F: Fn(X::O, X::O) -> X::O,
    {
        super::reduce::reduce(self, state, iter, x, f)
    }
}
