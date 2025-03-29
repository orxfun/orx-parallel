use crate::computations::Values;
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;

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

    fn new_run<I, T>(mut self, iter: &I, shared_state: &Self::SharedState, task: &T)
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

    fn new_run_with_idx<I, T>(mut self, iter: &I, shared_state: &Self::SharedState, task: &T)
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

    fn new_collect_with_idx<I, T, Vt, O, Vo, M1, F, M2>(
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

    // zzzzzzzzzzzz

    fn run<I, T>(mut self, iter: &I, shared_state: &Self::SharedState, transform: &T)
    where
        I: ConcurrentIter,
        T: Fn(I::Item),
    {
        let mut chunk_puller = iter.chunk_puller(0);
        let mut item_puller = iter.item_puller();

        loop {
            let chunk_size = self.next_chunk_size(shared_state, iter);

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

                    match chunk_puller.pull() {
                        Some(chunk) => {
                            for value in chunk {
                                transform(value);
                            }
                        }
                        None => break,
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

        loop {
            let chunk_size = self.next_chunk_size(shared_state, iter);

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

                    match chunk_puller.pull_with_idx() {
                        Some((begin_idx, chunk)) => {
                            for (i, value) in chunk.enumerate() {
                                transform((begin_idx + i, value));
                            }
                        }
                        None => break,
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

        loop {
            let chunk_size = self.next_chunk_size(shared_state, iter);

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

                    match chunk_puller.pull_with_idx() {
                        Some((begin_idx, chunk)) => {
                            for (i, input) in chunk.enumerate() {
                                let intermediate = map1(input);
                                if filter(&intermediate) {
                                    out_vec.push((begin_idx + i, map2(intermediate)));
                                }
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

    fn mfm_collect_heap<I, T, Vt, O, Vo, M1, F, M2>(
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

    fn m_collect_in_order<I, O, M1, P>(
        mut self,
        iter: &I,
        shared_state: &Self::SharedState,
        map1: &M1,
        o_bag: &ConcurrentOrderedBag<O, P>,
    ) where
        I: ConcurrentIter,
        O: Send + Sync,
        M1: Fn(I::Item) -> O + Send + Sync,
        P: IntoConcurrentPinnedVec<O>,
    {
        // let mut collected = Vec::new();
        // let out_vec = &mut collected;

        // let mut chunk_puller = iter.chunk_puller(0);
        // let mut item_puller = iter.item_puller_with_idx();

        // while let Some(chunk_size) = self.next_chunk_size(shared_state, iter) {
        //     self.begin_chunk(chunk_size);

        //     match chunk_size {
        //         0 | 1 => match item_puller.next() {
        //             Some((i_idx, i)) => {
        //                 let vt = map1(i);
        //                 vt.mfm_collect_heap(i_idx, filter, map2, out_vec);
        //             }
        //             None => break,
        //         },
        //         c => {
        //             if c > chunk_puller.chunk_size() {
        //                 chunk_puller = iter.chunk_puller(c);
        //             }

        //             if let Some((chunk_begin_idx, chunk)) = chunk_puller.pull_with_idx() {
        //                 for (i_idx, i) in chunk.enumerate() {
        //                     let i_idx = chunk_begin_idx + i_idx;
        //                     let vt = map1(i);
        //                     vt.mfm_collect_heap(i_idx, filter, map2, out_vec);
        //                 }
        //             }
        //         }
        //     }

        //     self.complete_chunk(shared_state, chunk_size);
        // }

        // self.complete_task(shared_state);
    }

    // fn work<I>(mut self, iter: &I, shared_state: &Self::SharedState)
    // where
    //     I: ConcurrentIter,
    // {
    //     let mut collected = Vec::new();
    //     let out_vec = &mut collected;

    //     let mut chunk_puller = iter.chunk_puller(0);
    //     let mut item_puller = iter.item_puller_with_idx();

    //     while let Some(chunk_size) = self.next_chunk_size_2(shared_state, iter) {
    //         self.begin_chunk(chunk_size);

    //         match chunk_size {
    //             0 | 1 => match item_puller.next() {
    //                 Some((i_idx, i)) => {
    //                     let vt = map1(i);
    //                     vt.mfm_collect_heap(i_idx, filter, map2, out_vec);
    //                 }
    //                 None => break,
    //             },
    //             c => {
    //                 if c > chunk_puller.chunk_size() {
    //                     chunk_puller = iter.chunk_puller(c);
    //                 }

    //                 if let Some((chunk_begin_idx, chunk)) = chunk_puller.pull_with_idx() {
    //                     for (i_idx, i) in chunk.enumerate() {
    //                         let i_idx = chunk_begin_idx + i_idx;
    //                         let vt = map1(i);
    //                         vt.mfm_collect_heap(i_idx, filter, map2, out_vec);
    //                     }
    //                 }
    //             }
    //         }

    //         self.complete_chunk(shared_state, chunk_size);
    //     }

    //     self.complete_task(shared_state);
    // }
}
