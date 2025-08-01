use orx_concurrent_iter::ConcurrentIter;

/// Thread runner responsible for executing the tasks assigned to the thread by the
/// parallel runner.
pub trait ThreadRunner: Sized {
    /// Type of the shared state among threads.
    type SharedState;

    /// Returns the next chunks size to be pulled from the input `iter` for the given
    /// current `shared_state`.
    fn next_chunk_size<I>(&self, shared_state: &Self::SharedState, iter: &I) -> usize
    where
        I: ConcurrentIter;

    /// Hook that will be called before starting to execute the chunk of the given `chunk_size`.
    fn begin_chunk(&mut self, chunk_size: usize);

    /// Hook that will be called after completing the chunk of the given `chunk_size`.
    /// The `shared_state` is also provided so that it can be updated to send information to the
    /// parallel runner and other thread runners.
    fn complete_chunk(&mut self, shared_state: &Self::SharedState, chunk_size: usize);

    /// Hook that will be called after completing the task.
    /// The `shared_state` is also provided so that it can be updated to send information to the
    /// parallel runner and other thread runners.
    fn complete_task(&mut self, shared_state: &Self::SharedState);
}
