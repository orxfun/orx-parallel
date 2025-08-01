use crate::{
    ThreadRunner,
    computations::Values,
    runner::{ParallelTask, thread_runner_compute::*},
};
#[cfg(test)]
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;

pub(crate) trait ThreadRunnerCompute: ThreadRunner {
    // run

    fn run<I, T>(self, iter: &I, shared_state: &Self::SharedState, task: &T)
    where
        I: ConcurrentIter,
        T: ParallelTask<Item = I::Item>,
    {
        tasks::run(self, iter, shared_state, task);
    }

    // m

    fn m_collect_ordered<I, O, M1, P>(
        self,
        iter: &I,
        shared_state: &Self::SharedState,
        map1: &M1,
        o_bag: &ConcurrentOrderedBag<O, P>,
        offset: usize,
    ) where
        I: ConcurrentIter,
        O: Send + Sync,
        M1: Fn(I::Item) -> O + Send + Sync,
        P: IntoConcurrentPinnedVec<O>,
    {
        collect_ordered::m_collect_ordered(self, iter, shared_state, map1, o_bag, offset);
    }

    #[cfg(test)]
    fn m_collect_in_arbitrary_order<I, O, M1, P>(
        self,
        iter: &I,
        shared_state: &Self::SharedState,
        map1: &M1,
        bag: &ConcurrentBag<O, P>,
    ) where
        I: ConcurrentIter,
        O: Send + Sync,
        M1: Fn(I::Item) -> O + Send + Sync,
        P: IntoConcurrentPinnedVec<O>,
    {
        collect_arbitrary::m(self, iter, shared_state, map1, bag);
    }

    // m - using

    fn using_m_collect_ordered<U, I, O, M1, P>(
        self,
        using: U,
        iter: &I,
        shared_state: &Self::SharedState,
        map1: &M1,
        o_bag: &ConcurrentOrderedBag<O, P>,
        offset: usize,
    ) where
        I: ConcurrentIter,
        O: Send + Sync,
        M1: Fn(&mut U, I::Item) -> O + Send + Sync,
        P: IntoConcurrentPinnedVec<O>,
    {
        collect_ordered::using_m_collect_ordered(
            self,
            using,
            iter,
            shared_state,
            map1,
            o_bag,
            offset,
        );
    }

    #[cfg(test)]
    fn using_m_collect_in_arbitrary_order<U, I, O, M1, P>(
        self,
        using: U,
        iter: &I,
        shared_state: &Self::SharedState,
        map1: &M1,
        bag: &ConcurrentBag<O, P>,
    ) where
        I: ConcurrentIter,
        O: Send + Sync,
        M1: Fn(&mut U, I::Item) -> O + Send + Sync,
        P: IntoConcurrentPinnedVec<O>,
    {
        collect_arbitrary::using_m(self, using, iter, shared_state, map1, bag);
    }

    // x

    fn x_collect_ordered<I, Vo, X1>(
        self,
        iter: &I,
        shared_state: &Self::SharedState,
        xap1: &X1,
    ) -> Vec<(usize, Vo::Item)>
    where
        I: ConcurrentIter,
        Vo: Values + Send + Sync,
        Vo::Item: Send + Sync,
        X1: Fn(I::Item) -> Vo + Send + Sync,
    {
        collect_ordered::x_collect_ordered(self, iter, shared_state, xap1)
    }

    #[cfg(test)]
    fn x_collect_in_arbitrary_order<I, Vo, M1, P>(
        self,
        iter: &I,
        shared_state: &Self::SharedState,
        map1: &M1,
        bag: &ConcurrentBag<Vo::Item, P>,
    ) where
        I: ConcurrentIter,
        Vo: Values + Send + Sync,
        Vo::Item: Send + Sync,
        M1: Fn(I::Item) -> Vo + Send + Sync,
        P: IntoConcurrentPinnedVec<Vo::Item>,
    {
        collect_arbitrary::x(self, iter, shared_state, map1, bag);
    }

    fn using_x_collect_ordered<U, I, Vo, X1>(
        self,
        using: U,
        iter: &I,
        shared_state: &Self::SharedState,
        xap1: &X1,
    ) -> Vec<(usize, Vo::Item)>
    where
        I: ConcurrentIter,
        Vo: Values + Send + Sync,
        Vo::Item: Send + Sync,
        X1: Fn(&mut U, I::Item) -> Vo + Send + Sync,
    {
        collect_ordered::using_x_collect_ordered(self, using, iter, shared_state, xap1)
    }

    #[cfg(test)]
    fn using_x_collect_in_arbitrary_order<U, I, Vo, M1, P>(
        self,
        using: U,
        iter: &I,
        shared_state: &Self::SharedState,
        xap1: &M1,
        bag: &ConcurrentBag<Vo::Item, P>,
    ) where
        I: ConcurrentIter,
        Vo: Values + Send + Sync,
        Vo::Item: Send + Sync,
        M1: Fn(&mut U, I::Item) -> Vo + Send + Sync,
        P: IntoConcurrentPinnedVec<Vo::Item>,
    {
        collect_arbitrary::using_x_collect_in_arbitrary_order(
            self,
            using,
            iter,
            shared_state,
            xap1,
            bag,
        );
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

    // next

    fn xfx_next<I, Vt, Vo, M1, F, M2>(
        mut self,
        iter: &I,
        shared_state: &Self::SharedState,
        map1: &M1,
        filter: &F,
        map2: &M2,
    ) -> Option<(usize, Vo::Item)>
    where
        I: ConcurrentIter,
        Vt: Values,
        Vo: Values,
        Vo::Item: Send + Sync,
        M1: Fn(I::Item) -> Vt + Send + Sync,
        F: Fn(&Vt::Item) -> bool + Send + Sync,
        M2: Fn(Vt::Item) -> Vo + Send + Sync,
    {
        let mut chunk_puller = iter.chunk_puller(0);
        let mut item_puller = iter.item_puller_with_idx();

        loop {
            let chunk_size = self.next_chunk_size(shared_state, iter);

            self.begin_chunk(chunk_size);

            match chunk_size {
                0 | 1 => match item_puller.next() {
                    Some((idx, i)) => {
                        let vt = map1(i);
                        if let Some(first) = vt.fx_next(filter, map2) {
                            iter.skip_to_end();
                            self.complete_chunk(shared_state, chunk_size);
                            self.complete_task(shared_state);
                            return Some((idx, first));
                        }
                    }
                    None => break,
                },
                c => {
                    if c > chunk_puller.chunk_size() {
                        chunk_puller = iter.chunk_puller(c);
                    }

                    match chunk_puller.pull_with_idx() {
                        Some((idx, chunk)) => {
                            for i in chunk {
                                let vt = map1(i);
                                if let Some(first) = vt.fx_next(filter, map2) {
                                    iter.skip_to_end();
                                    self.complete_chunk(shared_state, chunk_size);
                                    self.complete_task(shared_state);
                                    return Some((idx, first));
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
        None
    }

    fn xfx_next_any<I, Vt, Vo, M1, F, M2>(
        mut self,
        iter: &I,
        shared_state: &Self::SharedState,
        map1: &M1,
        filter: &F,
        map2: &M2,
    ) -> Option<Vo::Item>
    where
        I: ConcurrentIter,
        Vt: Values,
        Vo: Values,
        Vo::Item: Send + Sync,
        M1: Fn(I::Item) -> Vt + Send + Sync,
        F: Fn(&Vt::Item) -> bool + Send + Sync,
        M2: Fn(Vt::Item) -> Vo + Send + Sync,
    {
        let mut chunk_puller = iter.chunk_puller(0);
        let mut item_puller = iter.item_puller();

        loop {
            let chunk_size = self.next_chunk_size(shared_state, iter);

            self.begin_chunk(chunk_size);

            match chunk_size {
                0 | 1 => match item_puller.next() {
                    Some(i) => {
                        let vt = map1(i);
                        let maybe_next = vt.fx_next(filter, map2);
                        if maybe_next.is_some() {
                            iter.skip_to_end();
                            self.complete_chunk(shared_state, chunk_size);
                            self.complete_task(shared_state);
                            return maybe_next;
                        }
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
                                let maybe_next = vt.fx_next(filter, map2);
                                if maybe_next.is_some() {
                                    iter.skip_to_end();
                                    self.complete_chunk(shared_state, chunk_size);
                                    self.complete_task(shared_state);
                                    return maybe_next;
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
        None
    }
}

impl<X: ThreadRunner> ThreadRunnerCompute for X {}
