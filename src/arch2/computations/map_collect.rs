use super::ParallelRunner;
use crate::{computations::computation_kind::ComputationKind, parameters::Params};
use orx_concurrent_iter::{ConcurrentIter, Enumerated};
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;

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

    pub fn compute<R: ParallelRunner>(self) -> (usize, ConcurrentOrderedBag<O, P>) {
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
                let initial_len = self.iter.try_get_len();
                let transform =
                    |(i, value)| unsafe { self.bag.set_value(offset + i, (self.map)(value)) };
                let runner = R::new(ComputationKind::Collect, self.params, initial_len);
                runner.run(&self.iter, &transform);
                (0, self.bag)
            }
        }
    }
}
