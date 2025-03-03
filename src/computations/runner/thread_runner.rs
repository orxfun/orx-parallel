use orx_concurrent_iter::{ChunkPuller, ConcurrentIter, Element, Enumeration};

pub trait ThreadRunner: Sized {
    type SharedState;

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
        let mut chunk_puller = iter.chunk_puller(0);
        let mut item_puller = iter.item_puller();

        while let Some(chunk_size) = self.next_chunk_size(shared_state, iter) {
            self.begin_chunk(chunk_size);

            match chunk_size {
                0 | 1 => match item_puller.next() {
                    Some(value) => transform(value),
                    None => break,
                },
                c => {
                    if c > chunk_puller.chunk_size() {
                        chunk_puller = iter.chunk_puller(c);
                    }

                    let mut flattened = chunk_puller.flattened();
                    for value in flattened.by_ref() {
                        transform(value);
                    }
                    chunk_puller = flattened.into_chunk_puller();
                }
            }

            self.complete_chunk(shared_state, chunk_size);
        }

        self.complete_task(shared_state);
    }
}
