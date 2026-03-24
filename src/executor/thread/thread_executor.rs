use crate::{executor::val_and_idx::ValIdx, xap::Xap};
use alloc::vec::Vec;
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;
use orx_pinned_vec::IntoConcurrentPinnedVec;

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

    fn collect_arbitrary<I, X, P>(
        &mut self,
        state: &Self::SharedState,
        iter: &I,
        x: X,
        bag: &ConcurrentBag<X::O, P>,
    ) where
        I: ConcurrentIter,
        X: Xap<I = I::Item>,
        P: IntoConcurrentPinnedVec<X::O>,
        X::O: Send,
    {
        super::collect_arbitrary::collect_arbitrary(self, state, iter, x, bag)
    }

    fn collect_ordered<I, X>(
        &mut self,
        state: &Self::SharedState,
        iter: &I,
        x: X,
    ) -> Vec<ValIdx<X::O>>
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item>,
    {
        super::collect_ordered::collect_ordered(self, state, iter, x)
    }

    fn next<I, X>(&mut self, state: &Self::SharedState, iter: &I, x: X) -> Option<ValIdx<X::O>>
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item>,
    {
        super::next::next(self, state, iter, x)
    }

    fn next_any<I, X>(&mut self, state: &Self::SharedState, iter: &I, x: X) -> Option<X::O>
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item>,
    {
        super::next_any::next_any(self, state, iter, x)
    }

    fn reduce<I, X, F>(&mut self, state: &Self::SharedState, iter: &I, x: X, f: F) -> Option<X::O>
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item>,
        F: Fn(X::O, X::O) -> X::O,
    {
        super::reduce::reduce(self, state, iter, x, f)
    }
}
