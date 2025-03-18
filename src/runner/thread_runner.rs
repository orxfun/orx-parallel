use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub trait ThreadRunner: Sized {
    type SharedState;

    fn next_chunk_size<I>(&self, shared_state: &Self::SharedState, iter: &I) -> Option<usize>
    where
        I: ConcurrentIter;

    fn begin_chunk(&mut self, chunk_size: usize);

    fn complete_chunk(&mut self, shared_state: &Self::SharedState, chunk_size: usize);

    fn complete_task(&mut self, shared_state: &Self::SharedState);

    // run

    fn run<I, T>(mut self, iter: &I, shared_state: &Self::SharedState, transform: &T)
    where
        I: ConcurrentIter,
        T: Fn(I::Item),
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

                    if let Some(chunk) = chunk_puller.pull() {
                        for value in chunk {
                            transform(value);
                        }
                    }
                }
            }

            self.complete_chunk(shared_state, chunk_size);
        }

        self.complete_task(shared_state);
    }

    fn run_with_idx<I, T>(mut self, iter: &I, shared_state: &Self::SharedState, transform: &T)
    where
        I: ConcurrentIter,
        T: Fn((usize, I::Item)),
    {
        let mut chunk_puller = iter.chunk_puller(0);
        let mut item_puller = iter.item_puller_with_idx();

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

                    if let Some((begin_idx, chunk)) = chunk_puller.pull_with_idx() {
                        for (i, value) in chunk.enumerate() {
                            transform((begin_idx + i, value));
                        }
                    }
                }
            }

            self.complete_chunk(shared_state, chunk_size);
        }

        self.complete_task(shared_state);
    }

    // run - into vec

    fn run_with_idx_into_vec<I, T, O>(
        mut self,
        iter: &I,
        shared_state: &Self::SharedState,
        transform: &T,
        capacity: Option<usize>,
    ) -> Vec<O>
    where
        I: ConcurrentIter,
        T: Fn((usize, I::Item)) -> O,
    {
        let mut output = new_vec(capacity);
        let mut chunk_puller = iter.chunk_puller(0);
        let mut item_puller = iter.item_puller_with_idx();

        while let Some(chunk_size) = self.next_chunk_size(shared_state, iter) {
            self.begin_chunk(chunk_size);

            match chunk_size {
                0 | 1 => match item_puller.next() {
                    Some(value) => output.push(transform(value)),
                    None => break,
                },
                c => {
                    if c > chunk_puller.chunk_size() {
                        chunk_puller = iter.chunk_puller(c);
                    }

                    if let Some((begin_idx, chunk)) = chunk_puller.pull_with_idx() {
                        for (i, value) in chunk.enumerate() {
                            let value = transform((begin_idx + i, value));
                            output.push(value);
                        }
                    }
                }
            }

            self.complete_chunk(shared_state, chunk_size);
        }

        self.complete_task(shared_state);
        output
    }
}

fn new_vec<T>(capacity: Option<usize>) -> Vec<T> {
    match capacity {
        Some(x) => Vec::with_capacity(x),
        None => Vec::new(),
    }
}
