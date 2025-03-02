use super::parallel_runner::ParallelRunner;
use crate::parameters::ChunkSize;
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter, Element, Enumeration};

pub trait ThreadRunner: Sized {
    type SharedState;

    type ParallelRunner: ParallelRunner<SharedState = Self::SharedState>;

    fn new(chunk_size: ChunkSize) -> Self;

    fn next_chunk_size<E, I>(&self, shared_state: &Self::SharedState, iter: &I) -> Option<usize>
    where
        E: Enumeration,
        I: ConcurrentIter<E>;

    fn begin_chunk(&mut self, chunk_size: usize);

    fn complete_chunk(&mut self, shared_state: &Self::SharedState, chunk_size: usize);

    fn complete_task(&mut self, shared_state: &Self::SharedState);

    fn run<E, I, T>(mut self, iter: &I, shared_state: &Self::SharedState, transform: &T)
    where
        E: Enumeration,
        I: ConcurrentIter<E>,
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
