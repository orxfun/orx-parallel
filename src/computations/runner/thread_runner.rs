use super::parallel_runner::ParallelRunner;
use crate::parameters::Params;
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter, Element, Enumeration};

type SharedState<T> = <<T as ThreadRunner>::ParallelRunner as ParallelRunner>::SharedState;

pub trait ThreadRunner: Sized {
    type ParallelRunner: ParallelRunner;

    fn new(params: Params) -> Self;

    fn next_chunk_size<I>(&self, shared_state: &SharedState<Self>, iter: &I) -> Option<usize>;

    fn begin_chunk(&mut self, chunk_size: usize);

    fn complete_chunk(&mut self, shared_state: &SharedState<Self>, chunk_size: usize);

    fn complete_task(&mut self, shared_state: &SharedState<Self>);

    fn run<E, I, T>(mut self, iter: &I, shared_state: &SharedState<Self>, transform: &T)
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
