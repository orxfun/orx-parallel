use crate::computations::Values;
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

use super::parallel_task::{ParallelTask, ParallelTaskWithIdx};

pub trait ThreadRunner: Sized {
    type SharedState;

    fn next_chunk_size<I>(&self, shared_state: &Self::SharedState, iter: &I) -> usize
    where
        I: ConcurrentIter;

    fn begin_chunk(&mut self, chunk_size: usize);

    fn complete_chunk(&mut self, shared_state: &Self::SharedState, chunk_size: usize);

    fn complete_task(&mut self, shared_state: &Self::SharedState);

    // run

    fn run<I, T>(mut self, iter: &I, shared_state: &Self::SharedState, task: &T)
    where
        I: ConcurrentIter,
        T: ParallelTask<Item = I::Item>,
    {
        let mut chunk_puller = iter.chunk_puller(0);
        let mut item_puller = iter.item_puller();

        loop {
            let chunk_size = self.next_chunk_size(shared_state, iter);

            self.begin_chunk(chunk_size);

            match chunk_size {
                0 | 1 => match item_puller.next() {
                    Some(value) => task.f1(value),
                    None => break,
                },
                c => {
                    if c > chunk_puller.chunk_size() {
                        chunk_puller = iter.chunk_puller(c);
                    }

                    match chunk_puller.pull() {
                        Some(chunk) => task.fc(chunk),
                        None => break,
                    }
                }
            }

            self.complete_chunk(shared_state, chunk_size);
        }

        self.complete_task(shared_state);
    }

    fn run_with_idx<I, T>(mut self, iter: &I, shared_state: &Self::SharedState, task: &T)
    where
        I: ConcurrentIter,
        T: ParallelTaskWithIdx<Item = I::Item>,
    {
        let mut chunk_puller = iter.chunk_puller(0);
        let mut item_puller = iter.item_puller_with_idx();

        loop {
            let chunk_size = self.next_chunk_size(shared_state, iter);

            self.begin_chunk(chunk_size);

            match chunk_size {
                0 | 1 => match item_puller.next() {
                    Some((idx, value)) => task.f1(idx, value),
                    None => break,
                },
                c => {
                    if c > chunk_puller.chunk_size() {
                        chunk_puller = iter.chunk_puller(c);
                    }

                    match chunk_puller.pull_with_idx() {
                        Some((begin_idx, chunk)) => task.fc(begin_idx, chunk),
                        None => break,
                    }
                }
            }

            self.complete_chunk(shared_state, chunk_size);
        }

        self.complete_task(shared_state);
    }

    fn mfm_collect_with_idx<I, Vt, Vo, M1, F, M2>(
        mut self,
        iter: &I,
        shared_state: &Self::SharedState,
        map1: &M1,
        filter: &F,
        map2: &M2,
    ) -> Vec<(usize, Vo::Item)>
    where
        I: ConcurrentIter,
        Vt: Values,
        Vo: Values,
        Vo::Item: Send + Sync,
        M1: Fn(I::Item) -> Vt + Send + Sync,
        F: Fn(&Vt::Item) -> bool + Send + Sync,
        M2: Fn(Vt::Item) -> Vo + Send + Sync,
    {
        let mut collected = Vec::new();
        let out_vec = &mut collected;

        let mut chunk_puller = iter.chunk_puller(0);
        let mut item_puller = iter.item_puller_with_idx();

        loop {
            let chunk_size = self.next_chunk_size(shared_state, iter);

            self.begin_chunk(chunk_size);

            match chunk_size {
                0 | 1 => match item_puller.next() {
                    Some((i_idx, i)) => {
                        let vt = map1(i);
                        vt.mfm_collect_heap(i_idx, filter, map2, out_vec);
                    }
                    None => break,
                },
                c => {
                    if c > chunk_puller.chunk_size() {
                        chunk_puller = iter.chunk_puller(c);
                    }

                    match chunk_puller.pull_with_idx() {
                        Some((chunk_begin_idx, chunk)) => {
                            for (i_idx, i) in chunk.enumerate() {
                                let i_idx = chunk_begin_idx + i_idx;
                                let vt = map1(i);
                                vt.mfm_collect_heap(i_idx, filter, map2, out_vec);
                            }
                        }
                        None => break,
                    }
                }
            }

            self.complete_chunk(shared_state, chunk_size);
        }

        self.complete_task(shared_state);
        collected
    }
}
