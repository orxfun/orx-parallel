use orx_concurrent_iter::{ChunkPuller, ConcurrentIter, Enumerated};

pub trait ThreadRunner: Sized {
    type SharedState;

    fn next_chunk_size<I>(&self, shared_state: &Self::SharedState, iter: &I) -> Option<usize>;

    fn begin_chunk(&mut self, chunk_size: usize);

    fn complete_chunk(&mut self, shared_state: &Self::SharedState, chunk_size: usize);

    fn run<I, T>(mut self, iter: &I, shared_state: &Self::SharedState, transform: &T)
    where
        I: ConcurrentIter,
        T: Fn(I::Item),
    {
        while let Some(chunk_size) = self.next_chunk_size(shared_state, iter) {
            self.begin_chunk(chunk_size);

            match chunk_size {
                0 | 1 => transform_chunk(iter.item_puller(), transform),
                c => transform_chunk(iter.chunk_puller(c).flattened(), transform),
            }

            self.complete_chunk(shared_state, chunk_size);
        }
    }

    fn run_enumerated<I, T>(mut self, iter: &I, shared_state: &Self::SharedState, transform: &T)
    where
        I: ConcurrentIter<Enumerated>,
        T: Fn((usize, I::Item)),
    {
        while let Some(chunk_size) = self.next_chunk_size(shared_state, iter) {
            self.begin_chunk(chunk_size);

            match chunk_size {
                0 | 1 => transform_chunk(iter.item_puller(), transform),
                c => transform_chunk(iter.chunk_puller(c).flattened(), transform),
            }

            self.complete_chunk(shared_state, chunk_size);
        }
    }
}

fn transform_chunk<P, T>(puller: P, transform: T)
where
    P: Iterator,
    T: Fn(P::Item),
{
    for value in puller {
        transform(value);
    }
}
