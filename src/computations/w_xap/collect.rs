use super::x::WithX;
use crate::{
    IterationOrder,
    computations::{Values, heap_sort::heap_sort_into},
    runner::{ComputationKind, ParallelRunner, ParallelRunnerCompute, ParallelTask},
};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::IntoConcurrentPinnedVec;
use std::marker::PhantomData;

// arbitrary

struct XCollectInArbitraryOrder<'a, I, T, Vo, M1, P>
where
    T: Send + Clone,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(&mut T, I) -> Vo + Send + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    xap1: &'a M1,
    with: T,
    bag: &'a ConcurrentBag<Vo::Item, P>,
    phantom: PhantomData<I>,
}

impl<'a, I, T, Vo, M1, P> Clone for XCollectInArbitraryOrder<'a, I, T, Vo, M1, P>
where
    T: Send + Clone,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(&mut T, I) -> Vo + Send + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    fn clone(&self) -> Self {
        Self {
            xap1: self.xap1,
            with: self.with.clone(),
            bag: self.bag,
            phantom: self.phantom,
        }
    }
}

impl<'a, I, T, Vo, M1, P> ParallelTask for XCollectInArbitraryOrder<'a, I, T, Vo, M1, P>
where
    T: Send + Clone,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(&mut T, I) -> Vo + Send + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    type Item = I;

    fn f1(&mut self, value: Self::Item) {
        let values_vt = (self.xap1)(&mut self.with, value);
        for x in values_vt.values() {
            self.bag.push(x);
        }
    }

    fn fc(&mut self, values: impl ExactSizeIterator<Item = Self::Item>) {
        for x in values {
            self.f1(x);
        }
    }
}
