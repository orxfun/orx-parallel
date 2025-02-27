use std::marker::PhantomData;

use crate::{
    computations::{computation_kind::ComputationKind, ParallelRunner},
    parameters::Params,
};
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter, Enumerated};
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;

pub struct MapCollect<I, O, Map, P, R>
where
    I: ConcurrentIter<Enumerated>,
    O: Send + Sync,
    Map: Fn(I::Item) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
    R: ParallelRunner,
{
    params: Params,
    iter: I,
    map: Map,
    bag: ConcurrentOrderedBag<O, P>,
    phantom: PhantomData<R>,
}

unsafe impl<I, O, Map, P, R> Sync for MapCollect<I, O, Map, P, R>
where
    I: ConcurrentIter<Enumerated>,
    O: Send + Sync,
    Map: Fn(I::Item) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
    R: ParallelRunner,
{
}

impl<I, O, Map, P, R> MapCollect<I, O, Map, P, R>
where
    I: ConcurrentIter<Enumerated>,
    O: Send + Sync,
    Map: Fn(I::Item) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
    R: ParallelRunner,
{
    pub fn run(params: Params, iter: I, map: Map, bag: ConcurrentOrderedBag<O, P>) {
        let map_collect = Self {
            params,
            iter,
            map,
            bag,
            phantom: PhantomData,
        };

        map_collect.compute();
    }

    fn compute(self) {
        match self.params.is_sequential() {
            true => {
                // # SAFETY: collected is just wrapped as a concurrent-ordered-bag and is not mutated by par-iters,
                // hence it is safe to convert it back to the underlying pinned vector.
                let mut vec = unsafe { self.bag.into_inner().unwrap_only_if_counts_match() };
                for x in self.iter.into_seq_iter().map(self.map) {
                    vec.push(x);
                }
            }
            false => {
                let len = self.iter.try_get_len();
                let runner = R::new(ComputationKind::Collect, self.params, len);
                let thread_task = |chunk_size| self.thread_task(chunk_size);
                runner.run(&self.iter, &thread_task);
            }
        }
    }

    pub fn thread_task(&self, chunk_size: usize)
    where
        I: ConcurrentIter<Enumerated>,
        O: Send + Sync,
        Map: Fn(I::Item) -> O + Send + Sync,
        P: IntoConcurrentPinnedVec<O>,
    {
        match chunk_size {
            0 | 1 => self.pull_into_bag(self.iter.item_puller()),
            c => self.pull_into_bag(self.iter.chunk_puller(c).flattened()),
        }
    }

    fn pull_into_bag<EI>(&self, puller: EI)
    where
        EI: Iterator<Item = (usize, I::Item)>,
    {
        for (i, value) in puller {
            unsafe { self.bag.set_value(i, (self.map)(value)) };
        }
    }
}
