use super::x::X;
use crate::{computations::Values, runner::ParallelTask};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::IntoConcurrentPinnedVec;
use std::marker::PhantomData;

pub struct XCollect<I, Vo, M1, P>
where
    I: ConcurrentIter,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vo + Send + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    x: X<I, Vo, M1>,
    pinned_vec: P,
}

// arbitrary

struct XCollectInArbitraryOrder<'a, I, Vo, M1, P>
where
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I) -> Vo + Send + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    map1: M1,
    bag: &'a ConcurrentBag<Vo::Item, P>,
    phantom: PhantomData<I>,
}

impl<'a, I, Vo, M1, P> XCollectInArbitraryOrder<'a, I, Vo, M1, P>
where
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I) -> Vo + Send + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    fn new(map1: M1, bag: &'a ConcurrentBag<Vo::Item, P>) -> Self {
        Self {
            map1,
            bag,
            phantom: PhantomData,
        }
    }
}

impl<'a, I, Vo, M1, P> ParallelTask for XCollectInArbitraryOrder<'a, I, Vo, M1, P>
where
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I) -> Vo + Send + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    type Item = I;

    #[inline]
    fn f1(&self, value: Self::Item) {
        let values_vt = (self.map1)(value);
        for x in values_vt.values() {
            self.bag.push(x);
        }
    }

    #[inline(always)]
    fn fc(&self, values: impl ExactSizeIterator<Item = Self::Item>) {
        for x in values {
            self.f1(x);
        }
    }
}
