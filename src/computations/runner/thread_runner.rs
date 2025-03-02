use super::parallel_runner::ParallelRunner;
use crate::{computations::computation_kind::ComputationKind, parameters::ChunkSize};
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter, Element, Enumeration};

pub trait ThreadRunner<E, I>: Sized
where
    E: Enumeration,
    I: ConcurrentIter<E>,
{
    type SharedState;

    type ParallelRunner: ParallelRunner<E, I, SharedState = Self::SharedState>;

    fn new() -> Self;

    fn next_chunk_size(&self, shared_state: &Self::SharedState, iter: &I) -> Option<usize>;

    fn begin_chunk(&mut self, chunk_size: usize);

    fn complete_chunk(&mut self, shared_state: &Self::SharedState, chunk_size: usize);

    fn complete_task(&mut self, shared_state: &Self::SharedState);

    fn run<T>(
        mut self,
        kind: ComputationKind,
        chunk_size: ChunkSize,
        iter: &I,
        shared_state: &Self::SharedState,
        transform: &T,
    ) where
        T: Fn(<E::Element as Element>::ElemOf<I::Item>),
    {
        while let Some(chunk_size) = self.next_chunk_size(shared_state, iter) {
            self.begin_chunk(chunk_size);

            match chunk_size {
                0 | 1 => transform_chunk(iter.item_puller(), transform),
                c => transform_chunk(iter.chunk_puller(c).flattened(), transform),
            }

            self.complete_chunk(shared_state, chunk_size);
        }

        self.complete_task(shared_state);
    }
}

fn transform_chunk<I, T>(puller: I, transform: T)
where
    I: Iterator,
    T: Fn(I::Item),
{
    for value in puller {
        transform(value);
    }
}
