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

impl<I, T, Vo, M1> WithX<I, T, Vo, M1>
where
    I: ConcurrentIter,
    T: Send + Sync + Clone,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(&mut T, I::Item) -> Vo + Clone + Send + Sync,
{
    pub fn collect_into<R, P>(self, pinned_vec: P) -> (usize, P)
    where
        R: ParallelRunner,
        P: IntoConcurrentPinnedVec<Vo::Item>,
    {
        let x_collect = XCollect {
            x: self,
            pinned_vec,
        };
        let params = x_collect.x.params();
        match (params.is_sequential(), params.iteration_order) {
            (true, _) => (0, x_collect.sequential()),
            (false, IterationOrder::Arbitrary) => x_collect.parallel_in_arbitrary::<R>(),
            (false, IterationOrder::Ordered) => x_collect.parallel_with_heap_sort::<R>(),
        }
    }
}

pub struct XCollect<I, T, Vo, M1, P>
where
    I: ConcurrentIter,
    T: Send + Sync + Clone,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(&mut T, I::Item) -> Vo + Clone + Send + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    x: WithX<I, T, Vo, M1>,
    pinned_vec: P,
}

impl<I, T, Vo, M1, P> XCollect<I, T, Vo, M1, P>
where
    I: ConcurrentIter,
    T: Send + Sync + Clone,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(&mut T, I::Item) -> Vo + Clone + Send + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    fn sequential(self) -> P {
        let (x, mut pinned_vec) = (self.x, self.pinned_vec);
        let (_, iter, mut with, xap1) = x.destruct();

        let iter = iter.into_seq_iter();
        for i in iter {
            let vt = xap1(&mut with, i);
            vt.push_to_pinned_vec(&mut pinned_vec);
        }

        pinned_vec
    }

    fn parallel_in_arbitrary<R: ParallelRunner>(self) -> (usize, P) {
        let (x, pinned_vec) = (self.x, self.pinned_vec);
        let (params, iter, with, xap1) = x.destruct();

        let capacity_bound = pinned_vec.capacity_bound();
        let mut bag: ConcurrentBag<Vo::Item, P> = pinned_vec.into();
        bag.reserve_maximum_capacity(capacity_bound);

        let task = XCollectInArbitraryOrder::<'_, I::Item, T, Vo, M1, P> {
            xap1: &xap1,
            with,
            bag: &bag,
            phantom: PhantomData,
        };
        let runner = R::new(ComputationKind::Collect, params, iter.try_get_len());
        let num_spawned = runner.run(&iter, task);

        let values = bag.into_inner();
        (num_spawned, values)
    }

    fn parallel_with_heap_sort<R: ParallelRunner>(self) -> (usize, P) {
        let (x, mut pinned_vec) = (self.x, self.pinned_vec);
        let (params, iter, with, xap1) = x.destruct();
        let initial_len = iter.try_get_len();

        let runner = R::new(ComputationKind::Collect, params, initial_len);

        let create_map = || {
            let xap1 = xap1.clone();
            let mut with = with.clone();
            move |value| (xap1)(&mut with, value)
        };
        let (num_spawned, vectors) = runner.x_collect_with_idx(&iter, create_map);
        heap_sort_into(vectors, &mut pinned_vec);
        (num_spawned, pinned_vec)
    }
}

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
