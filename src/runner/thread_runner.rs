use super::parallel_task::{ParallelTask, ParallelTaskWithIdx};
use crate::computations::Values;
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub trait ThreadRunner: Sized {
    type SharedState;

    fn next_chunk_size<I>(&self, shared_state: &Self::SharedState, iter: &I) -> usize
    where
        I: ConcurrentIter;

    fn begin_chunk(&mut self, chunk_size: usize);

    fn complete_chunk(&mut self, shared_state: &Self::SharedState, chunk_size: usize);

    fn complete_task(&mut self, shared_state: &Self::SharedState);
}

pub(crate) trait ThreadRunnerCompute: ThreadRunner {
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

    // collect

    fn x_collect_with_idx<I, Vo, M1>(
        mut self,
        iter: &I,
        shared_state: &Self::SharedState,
        map1: &M1,
    ) -> Vec<(usize, Vo::Item)>
    where
        I: ConcurrentIter,
        Vo: Values,
        Vo::Item: Send + Sync,
        M1: Fn(I::Item) -> Vo + Send + Sync,
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
                    Some((idx, i)) => {
                        let vo = map1(i);
                        vo.push_to_vec_with_idx(idx, out_vec);
                    }
                    None => break,
                },
                c => {
                    if c > chunk_puller.chunk_size() {
                        chunk_puller = iter.chunk_puller(c);
                    }

                    match chunk_puller.pull_with_idx() {
                        Some((chunk_begin_idx, chunk)) => {
                            for i in chunk {
                                let vo = map1(i);
                                vo.push_to_vec_with_idx(chunk_begin_idx, out_vec);
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

    fn xfx_collect_with_idx<I, Vt, Vo, M1, F, M2>(
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
                        vt.xfx_collect_heap(i_idx, filter, map2, out_vec);
                    }
                    None => break,
                },
                c => {
                    if c > chunk_puller.chunk_size() {
                        chunk_puller = iter.chunk_puller(c);
                    }

                    match chunk_puller.pull_with_idx() {
                        Some((chunk_begin_idx, chunk)) => {
                            for i in chunk {
                                let vt = map1(i);
                                vt.xfx_collect_heap(chunk_begin_idx, filter, map2, out_vec);
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

    // reduce

    fn x_reduce<I, Vo, M1, X>(
        mut self,
        iter: &I,
        shared_state: &Self::SharedState,
        map1: &M1,
        reduce: &X,
    ) -> Option<Vo::Item>
    where
        I: ConcurrentIter,
        Vo: Values,
        Vo::Item: Send + Sync,
        M1: Fn(I::Item) -> Vo + Send + Sync,
        X: Fn(Vo::Item, Vo::Item) -> Vo::Item + Send + Sync,
    {
        let mut chunk_puller = iter.chunk_puller(0);
        let mut item_puller = iter.item_puller();

        let mut acc = None;
        loop {
            let chunk_size = self.next_chunk_size(shared_state, iter);

            self.begin_chunk(chunk_size);

            match chunk_size {
                0 | 1 => match item_puller.next() {
                    Some(i) => {
                        let vo = map1(i);
                        acc = vo.acc_reduce(acc, reduce);
                    }
                    None => break,
                },
                c => {
                    if c > chunk_puller.chunk_size() {
                        chunk_puller = iter.chunk_puller(c);
                    }

                    match chunk_puller.pull() {
                        Some(chunk) => {
                            for i in chunk {
                                let vo = map1(i);
                                acc = vo.acc_reduce(acc, reduce);
                            }
                        }
                        None => break,
                    }
                }
            }

            self.complete_chunk(shared_state, chunk_size);
        }

        self.complete_task(shared_state);
        acc
    }

    fn xfx_reduce<I, Vt, Vo, M1, F, M2, X>(
        mut self,
        iter: &I,
        shared_state: &Self::SharedState,
        map1: &M1,
        filter: &F,
        map2: &M2,
        reduce: &X,
    ) -> Option<Vo::Item>
    where
        I: ConcurrentIter,
        Vt: Values,
        Vo: Values,
        Vo::Item: Send + Sync,
        M1: Fn(I::Item) -> Vt + Send + Sync,
        F: Fn(&Vt::Item) -> bool + Send + Sync,
        M2: Fn(Vt::Item) -> Vo + Send + Sync,
        X: Fn(Vo::Item, Vo::Item) -> Vo::Item + Send + Sync,
    {
        let mut chunk_puller = iter.chunk_puller(0);
        let mut item_puller = iter.item_puller();

        let mut acc = None;

        loop {
            let chunk_size = self.next_chunk_size(shared_state, iter);

            self.begin_chunk(chunk_size);

            match chunk_size {
                0 | 1 => match item_puller.next() {
                    Some(i) => {
                        let vt = map1(i);
                        acc = vt.fx_reduce(acc, filter, map2, reduce);
                    }
                    None => break,
                },
                c => {
                    if c > chunk_puller.chunk_size() {
                        chunk_puller = iter.chunk_puller(c);
                    }

                    match chunk_puller.pull() {
                        Some(chunk) => {
                            for i in chunk {
                                let vt = map1(i);
                                acc = vt.fx_reduce(acc, filter, map2, reduce);
                            }
                        }
                        None => break,
                    }
                }
            }

            self.complete_chunk(shared_state, chunk_size);
        }

        self.complete_task(shared_state);
        acc
    }

    // first

    // fn xfx_first<I, Vt, Vo, M1, F, M2>(
    //     mut self,
    //     iter: &I,
    //     shared_state: &Self::SharedState,
    //     map1: &M1,
    //     filter: &F,
    //     map2: &M2,
    // ) -> Option<Vo::Item>
    // where
    //     I: ConcurrentIter,
    //     Vt: Values,
    //     Vo: Values,
    //     Vo::Item: Send + Sync,
    //     M1: Fn(I::Item) -> Vt + Send + Sync,
    //     F: Fn(&Vt::Item) -> bool + Send + Sync,
    //     M2: Fn(Vt::Item) -> Vo + Send + Sync,
    // {
    //     let mut chunk_puller = iter.chunk_puller(0);
    //     let mut item_puller = iter.item_puller();

    //     let mut acc = None;

    //     loop {
    //         let chunk_size = self.next_chunk_size(shared_state, iter);

    //         self.begin_chunk(chunk_size);

    //         match chunk_size {
    //             0 | 1 => match item_puller.next() {
    //                 Some(i) => {
    //                     let vt = map1(i);
    //                     acc = vt.fx_reduce(acc, filter, map2, reduce);
    //                 }
    //                 None => break,
    //             },
    //             c => {
    //                 if c > chunk_puller.chunk_size() {
    //                     chunk_puller = iter.chunk_puller(c);
    //                 }

    //                 match chunk_puller.pull() {
    //                     Some(chunk) => {
    //                         for i in chunk {
    //                             let vt = map1(i);
    //                             acc = vt.fx_reduce(acc, filter, map2, reduce);
    //                         }
    //                     }
    //                     None => break,
    //                 }
    //             }
    //         }

    //         self.complete_chunk(shared_state, chunk_size);
    //     }

    //     self.complete_task(shared_state);
    //     acc
    // }
}

impl<X: ThreadRunner> ThreadRunnerCompute for X {}
