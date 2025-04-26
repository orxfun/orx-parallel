use super::xfx::Xfx;
use crate::CollectOrdering;
use crate::computations::Values;
use crate::computations::heap_sort::heap_sort_into;
use crate::runner::{ComputationKind, ParallelRunner, ParallelRunnerCompute, ParallelTask};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;
use orx_pinned_vec::IntoConcurrentPinnedVec;
use std::marker::PhantomData;

impl<I, Vt, Vo, M1, F, M2> Xfx<I, Vt, Vo, M1, F, M2>
where
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&Vt::Item) -> bool + Send + Sync,
    M2: Fn(Vt::Item) -> Vo + Send + Sync,
{
    pub fn collect_into<R, P>(self, pinned_vec: P) -> (usize, P)
    where
        R: ParallelRunner,
        P: IntoConcurrentPinnedVec<Vo::Item>,
    {
        XfxCollect::compute::<R>(self, pinned_vec)
    }
}

pub struct XfxCollect<I, Vt, Vo, M1, F, M2, P>
where
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&Vt::Item) -> bool + Send + Sync,
    M2: Fn(Vt::Item) -> Vo + Send + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    xfx: Xfx<I, Vt, Vo, M1, F, M2>,
    pinned_vec: P,
}

impl<I, Vt, Vo, M1, F, M2, P> XfxCollect<I, Vt, Vo, M1, F, M2, P>
where
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&Vt::Item) -> bool + Send + Sync,
    M2: Fn(Vt::Item) -> Vo + Send + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    pub fn compute<R: ParallelRunner>(xfx: Xfx<I, Vt, Vo, M1, F, M2>, pinned_vec: P) -> (usize, P) {
        let xfx_collect = Self { xfx, pinned_vec };
        let params = xfx_collect.xfx.params();
        match (params.is_sequential(), params.collect_ordering) {
            (true, _) => (0, xfx_collect.sequential()),
            (false, CollectOrdering::Arbitrary) => xfx_collect.parallel_in_arbitrary::<R>(),
            (false, CollectOrdering::Ordered) => xfx_collect.parallel_with_heap_sort::<R>(),
        }
    }

    fn sequential(self) -> P {
        let (xfx, mut pinned_vec) = (self.xfx, self.pinned_vec);
        let (_, iter, xap1, filter, xap2) = xfx.destruct();

        let iter = iter.into_seq_iter();
        for i in iter {
            let vt = xap1(i);
            vt.filter_map_collect_sequential(&filter, &xap2, &mut pinned_vec);
        }

        pinned_vec
    }

    fn parallel_in_arbitrary<R: ParallelRunner>(self) -> (usize, P) {
        let (xfx, pinned_vec) = (self.xfx, self.pinned_vec);
        let (params, iter, xap1, filter, xap2) = xfx.destruct();

        let capacity_bound = pinned_vec.capacity_bound();
        let mut bag: ConcurrentBag<Vo::Item, P> = pinned_vec.into();
        bag.reserve_maximum_capacity(capacity_bound);

        let task = XfxCollectInArbitraryOrder::<'_, I, Vt, Vo, M1, F, M2, P>::new(
            xap1, filter, xap2, &bag,
        );

        let runner = R::new(ComputationKind::Collect, params, iter.try_get_len());
        let num_spawned = runner.run(&iter, task);

        let values = bag.into_inner();
        (num_spawned, values)
    }

    fn parallel_with_heap_sort<R: ParallelRunner>(self) -> (usize, P) {
        let (xfx, mut pinned_vec) = (self.xfx, self.pinned_vec);
        let (params, iter, map1, filter, map2) = xfx.destruct();
        let initial_len = iter.try_get_len();

        let runner = R::new(ComputationKind::Collect, params, initial_len);

        let (num_spawned, vectors) = runner.xfx_collect_with_idx(&iter, &map1, &filter, &map2);
        heap_sort_into(vectors, &mut pinned_vec);
        (num_spawned, pinned_vec)
    }
}

// arbitrary

struct XfxCollectInArbitraryOrder<'a, I, Vt, Vo, M1, F, M2, P>
where
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&Vt::Item) -> bool + Send + Sync,
    M2: Fn(Vt::Item) -> Vo + Send + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    map1: M1,
    filter: F,
    map2: M2,
    bag: &'a ConcurrentBag<Vo::Item, P>,
    phantom: PhantomData<(I, Vt, Vo)>,
}

impl<'a, I, Vt, Vo, M1, F, M2, P> XfxCollectInArbitraryOrder<'a, I, Vt, Vo, M1, F, M2, P>
where
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&Vt::Item) -> bool + Send + Sync,
    M2: Fn(Vt::Item) -> Vo + Send + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    fn new(map1: M1, filter: F, map2: M2, bag: &'a ConcurrentBag<Vo::Item, P>) -> Self {
        Self {
            map1,
            filter,
            map2,
            bag,
            phantom: PhantomData,
        }
    }
}

impl<I, Vt, Vo, M1, F, M2, P> ParallelTask
    for XfxCollectInArbitraryOrder<'_, I, Vt, Vo, M1, F, M2, P>
where
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&Vt::Item) -> bool + Send + Sync,
    M2: Fn(Vt::Item) -> Vo + Send + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    type Item = I::Item;

    #[inline]
    fn f1(&self, value: Self::Item) {
        let values_vt = (self.map1)(value);
        values_vt.filter_map_collect_arbitrary(&self.filter, &self.map2, self.bag);
    }

    #[inline(always)]
    fn fc(&self, values: impl ExactSizeIterator<Item = Self::Item>) {
        for x in values {
            self.f1(x);
        }
    }
}
