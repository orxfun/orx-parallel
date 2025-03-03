use crate::{
    computations::{computation_kind::ComputationKind, ParallelRunnerToArchive},
    parameters::Params,
};
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter, Enumerated};
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;
use rayon::iter::Enumerate;

use super::ParallelRunner;

pub struct MapCollect<I, O, Map, P>
where
    I: ConcurrentIter<Enumerated>,
    O: Send + Sync,
    Map: Fn(I::Item) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    params: Params,
    iter: I,
    map: Map,
    bag: ConcurrentOrderedBag<O, P>,
}

unsafe impl<I, O, Map, P> Sync for MapCollect<I, O, Map, P>
where
    I: ConcurrentIter<Enumerated>,
    O: Send + Sync,
    Map: Fn(I::Item) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
}

impl<I, O, Map, P> MapCollect<I, O, Map, P>
where
    I: ConcurrentIter<Enumerated>,
    O: Send + Sync,
    Map: Fn(I::Item) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    pub fn new<J>(params: Params, iter: J, map: Map, bag: ConcurrentOrderedBag<O, P>) -> Self
    where
        J: ConcurrentIter<EnumerationOf<Enumerated> = I>,
    {
        Self {
            params,
            iter: iter.enumerated(),
            map,
            bag,
        }
    }

    pub fn compute<R: ParallelRunner<Enumerated, I>>(self) -> (usize, ConcurrentOrderedBag<O, P>) {
        match self.params.is_sequential() {
            true => {
                // # SAFETY: collected is just wrapped as a concurrent-ordered-bag and is not mutated by par-iters,
                // hence it is safe to convert it back to the underlying pinned vector.
                let mut vec = unsafe { self.bag.into_inner().unwrap_only_if_counts_match() };
                for x in self.iter.into_seq_iter().map(self.map) {
                    vec.push(x);
                }
                (0, vec.into())
            }
            false => {
                let offset = self.bag.len();
                let transform =
                    |(i, value)| unsafe { self.bag.set_value(offset + i, (self.map)(value)) };
                let runner = R::new(ComputationKind::Collect, self.params, &self.iter);
                runner.run(&self.iter, &transform);
                (0, self.bag)
            }
        }
    }

    pub fn compute_to_arch<R: ParallelRunnerToArchive>(
        self,
    ) -> (usize, ConcurrentOrderedBag<O, P>) {
        match self.params.is_sequential() {
            true => {
                // # SAFETY: collected is just wrapped as a concurrent-ordered-bag and is not mutated by par-iters,
                // hence it is safe to convert it back to the underlying pinned vector.
                let mut vec = unsafe { self.bag.into_inner().unwrap_only_if_counts_match() };
                for x in self.iter.into_seq_iter().map(self.map) {
                    vec.push(x);
                }
                (0, vec.into())
            }
            false => {
                let offset = self.bag.len();
                let len = self.iter.try_get_len();
                let runner = R::new(ComputationKind::Collect, self.params, len);
                let thread_task = |chunk_size| self.thread_task(offset, chunk_size);
                let num_spawned = runner.run(&self.iter, &thread_task);
                (num_spawned, self.bag)
            }
        }
    }

    pub fn thread_task(&self, offset: usize, chunk_size: usize) {
        match chunk_size {
            0 | 1 => self.pull_into_bag(offset, self.iter.item_puller()),
            c => self.pull_into_bag(offset, self.iter.chunk_puller(c).flattened()),
        }
    }

    fn pull_into_bag<EI>(&self, offset: usize, puller: EI)
    where
        EI: Iterator<Item = (usize, I::Item)>,
    {
        for (i, value) in puller {
            unsafe { self.bag.set_value(offset + i, (self.map)(value)) };
        }
    }
}
