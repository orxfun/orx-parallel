use crate::computations::Values;
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

    fn mfm_collect_with_idx<I, T, O, M1, F, M2>(
        mut self,
        iter: &I,
        shared_state: &Self::SharedState,
        map1: &M1,
        filter: &F,
        map2: &M2,
    ) -> Vec<(usize, O)>
    where
        I: ConcurrentIter,
        M1: Fn(I::Item) -> T,
        F: Fn(&T) -> bool,
        M2: Fn(T) -> O,
    {
        let mut collected = Vec::new();
        let out_vec = &mut collected;

        let mut chunk_puller = iter.chunk_puller(0);
        let mut item_puller = iter.item_puller_with_idx();

        while let Some(chunk_size) = self.next_chunk_size(shared_state, iter) {
            self.begin_chunk(chunk_size);

            match chunk_size {
                0 | 1 => match item_puller.next() {
                    Some((i, input)) => {
                        let intermediate = map1(input);
                        if filter(&intermediate) {
                            out_vec.push((i, map2(intermediate)));
                        }
                    }
                    None => break,
                },
                c => {
                    if c > chunk_puller.chunk_size() {
                        chunk_puller = iter.chunk_puller(c);
                    }

                    if let Some((begin_idx, chunk)) = chunk_puller.pull_with_idx() {
                        for (i, input) in chunk.enumerate() {
                            let intermediate = map1(input);
                            if filter(&intermediate) {
                                out_vec.push((begin_idx + i, map2(intermediate)));
                            }
                        }
                    }
                }
            }

            self.complete_chunk(shared_state, chunk_size);
        }

        self.complete_task(shared_state);
        collected
    }

    fn mfm_collect<I, T, Vt, O, Vo, M1, F, M2>(
        mut self,
        iter: &I,
        shared_state: &Self::SharedState,
        map1: &M1,
        filter: &F,
        map2: &M2,
    ) -> Vec<(usize, O)>
    where
        I: ConcurrentIter,
        Vt: Values<Item = T>,
        O: Send + Sync,
        Vo: Values<Item = O>,
        M1: Fn(I::Item) -> Vt + Send + Sync,
        F: Fn(&T) -> bool + Send + Sync,
        M2: Fn(T) -> Vo + Send + Sync,
    {
        let mut collected = Vec::new();
        let out_vec = &mut collected;

        let mut chunk_puller = iter.chunk_puller(0);
        let mut item_puller = iter.item_puller_with_idx();

        while let Some(chunk_size) = self.next_chunk_size(shared_state, iter) {
            self.begin_chunk(chunk_size);

            match chunk_size {
                0 | 1 => match item_puller.next() {
                    Some((i_idx, i)) => {
                        let vt = map1(i);
                        vt.filter_map_collect_heap(i_idx, filter, map2, out_vec);
                    }
                    None => break,
                },
                c => {
                    if c > chunk_puller.chunk_size() {
                        chunk_puller = iter.chunk_puller(c);
                    }

                    if let Some((chunk_begin_idx, chunk)) = chunk_puller.pull_with_idx() {
                        for (i_idx, i) in chunk.enumerate() {
                            let i_idx = chunk_begin_idx + i_idx;
                            let vt = map1(i);
                            vt.filter_map_collect_heap(i_idx, filter, map2, out_vec);
                        }
                    }
                }
            }

            self.complete_chunk(shared_state, chunk_size);
        }

        self.complete_task(shared_state);
        collected
    }

    fn collect_into_vec_with_idx<I, T, O, M1, F, M2>(
        mut self,
        iter: &I,
        shared_state: &Self::SharedState,
        map1: &M1,
        filter: &F,
        map2: &M2,
        capacity: Option<usize>,
    ) -> Vec<(usize, O)>
    where
        I: ConcurrentIter,
        M1: Fn(I::Item) -> T,
        F: Fn(&T) -> bool,
        M2: Fn(T) -> O,
    {
        let mut collected = new_vec(capacity);
        let out_vec = &mut collected;

        let mut chunk_puller = iter.chunk_puller(0);
        let mut item_puller = iter.item_puller_with_idx();

        while let Some(chunk_size) = self.next_chunk_size(shared_state, iter) {
            self.begin_chunk(chunk_size);

            match chunk_size {
                0 | 1 => match item_puller.next() {
                    Some((i, input)) => {
                        let intermediate = map1(input);
                        if filter(&intermediate) {
                            out_vec.push((i, map2(intermediate)));
                        }
                    }
                    None => break,
                },
                c => {
                    if c > chunk_puller.chunk_size() {
                        chunk_puller = iter.chunk_puller(c);
                    }

                    if let Some((begin_idx, chunk)) = chunk_puller.pull_with_idx() {
                        for (i, input) in chunk.enumerate() {
                            let intermediate = map1(input);
                            if filter(&intermediate) {
                                out_vec.push((begin_idx + i, map2(intermediate)));
                            }
                        }
                    }
                }
            }

            self.complete_chunk(shared_state, chunk_size);
        }

        self.complete_task(shared_state);
        collected
    }

    fn collect_into_vec_with_idx2<I, O, M, F>(
        mut self,
        iter: &I,
        shared_state: &Self::SharedState,
        map: &M,
        filter: &F,
        capacity: Option<usize>,
    ) -> Vec<(usize, O)>
    where
        I: ConcurrentIter,
        M: Fn(I::Item) -> O,
        F: Fn(&O) -> bool,
    {
        let mut collected = new_vec(capacity);
        let out_vec = &mut collected;

        let mut chunk_puller = iter.chunk_puller(0);
        let mut item_puller = iter.item_puller_with_idx();

        while let Some(chunk_size) = self.next_chunk_size(shared_state, iter) {
            self.begin_chunk(chunk_size);

            match chunk_size {
                0 | 1 => match item_puller.next() {
                    Some((i, input)) => push_if_in(out_vec, i, input, map, filter),
                    None => break,
                },
                c => {
                    if c > chunk_puller.chunk_size() {
                        chunk_puller = iter.chunk_puller(c);
                    }

                    if let Some((begin_idx, chunk)) = chunk_puller.pull_with_idx() {
                        for (i, input) in chunk.enumerate() {
                            push_if_in(out_vec, begin_idx + i, input, map, filter);
                        }
                    }
                }
            }

            self.complete_chunk(shared_state, chunk_size);
        }

        self.complete_task(shared_state);
        collected
    }
}

#[inline(always)]
fn push_if_in<T, O, M, F>(out_vec: &mut Vec<(usize, O)>, i: usize, input: T, map: M, filter: F)
where
    M: Fn(T) -> O,
    F: Fn(&O) -> bool,
{
    let output = map(input);
    if filter(&output) {
        out_vec.push((i, output))
    }
}

fn new_vec<T>(capacity: Option<usize>) -> Vec<T> {
    match capacity {
        Some(x) => Vec::with_capacity(x),
        None => Vec::new(),
    }
}
