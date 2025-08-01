use super::m::M;
use crate::runner::{ComputationKind, ParallelRunner, ParallelRunnerCompute, ParallelTaskWithIdx};
#[cfg(test)]
use crate::{IterationOrder, runner::ParallelTask};
#[cfg(test)]
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_pinned_vec::IntoConcurrentPinnedVec;
use std::marker::PhantomData;

impl<I, O, M1> M<I, O, M1>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(I::Item) -> O + Send + Sync,
{
    pub fn collect_into<R, P>(self, pinned_vec: P) -> (usize, P)
    where
        R: ParallelRunner,
        P: IntoConcurrentPinnedVec<O>,
    {
        MCollect::compute::<R>(self, pinned_vec)
    }
}

pub struct MCollect<I, O, M1, P>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(I::Item) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    m: M<I, O, M1>,
    pinned_vec: P,
}

impl<I, O, M1, P> MCollect<I, O, M1, P>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(I::Item) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    pub fn compute<R: ParallelRunner>(m: M<I, O, M1>, pinned_vec: P) -> (usize, P) {
        let x = Self { m, pinned_vec };
        let p = x.m.params();
        match (p.is_sequential(), p.iteration_order) {
            (true, _) => (0, x.sequential()),
            #[cfg(test)]
            (false, IterationOrder::Arbitrary) => x.parallel_in_arbitrary_order::<R>(),
            (false, _) => x.parallel_in_input_order::<R>(),
        }
    }

    fn sequential(self) -> P {
        let (m, mut pinned_vec) = (self.m, self.pinned_vec);
        let (_, iter, map1) = m.destruct();

        let iter = iter.into_seq_iter();
        for i in iter {
            pinned_vec.push(map1(i));
        }

        pinned_vec
    }

    fn parallel_in_input_order<R: ParallelRunner>(self) -> (usize, P) {
        let (m, pinned_vec) = (self.m, self.pinned_vec);
        let runner = R::new(ComputationKind::Collect, m.params(), m.iter().try_get_len());
        runner.m_collect_ordered(m, pinned_vec)
    }

    #[cfg(test)]
    fn parallel_in_arbitrary_order<R: ParallelRunner>(self) -> (usize, P) {
        let (m, pinned_vec) = (self.m, self.pinned_vec);
        let runner = R::new(ComputationKind::Collect, m.params(), m.iter().try_get_len());
        runner.m_collect_in_arbitrary_order(m, pinned_vec)
    }
}

// ordered

struct MCollectInInputOrder<'a, I, O, M1, P>
where
    O: Send + Sync,
    M1: Fn(I) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    offset: usize,
    o_bag: &'a ConcurrentOrderedBag<O, P>,
    map1: M1,
    phantom: PhantomData<I>,
}

impl<'a, I, O, M1, P> MCollectInInputOrder<'a, I, O, M1, P>
where
    O: Send + Sync,
    M1: Fn(I) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    fn new(offset: usize, o_bag: &'a ConcurrentOrderedBag<O, P>, map1: M1) -> Self {
        Self {
            offset,
            o_bag,
            map1,
            phantom: PhantomData,
        }
    }
}

impl<I, O, M1, P> ParallelTaskWithIdx for MCollectInInputOrder<'_, I, O, M1, P>
where
    O: Send + Sync,
    M1: Fn(I) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    type Item = I;

    fn f1(&self, idx: usize, value: Self::Item) {
        unsafe { self.o_bag.set_value(self.offset + idx, (self.map1)(value)) };
    }

    fn fc(&self, begin_idx: usize, values: impl ExactSizeIterator<Item = Self::Item>) {
        let values = values.map(&self.map1);
        unsafe { self.o_bag.set_values(self.offset + begin_idx, values) };
    }
}

// in arbitrary order

#[cfg(test)]
struct MCollectInArbitraryOrder<'a, I, O, M1, P>
where
    O: Send + Sync,
    M1: Fn(I) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    bag: &'a ConcurrentBag<O, P>,
    map1: M1,
    phantom: PhantomData<I>,
}

#[cfg(test)]
impl<'a, I, O, M1, P> MCollectInArbitraryOrder<'a, I, O, M1, P>
where
    O: Send + Sync,
    M1: Fn(I) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    fn new(bag: &'a ConcurrentBag<O, P>, map1: M1) -> Self {
        Self {
            bag,
            map1,
            phantom: PhantomData,
        }
    }
}

#[cfg(test)]
impl<I, O, M1, P> ParallelTask for MCollectInArbitraryOrder<'_, I, O, M1, P>
where
    O: Send + Sync,
    M1: Fn(I) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    type Item = I;

    fn f1(&self, value: Self::Item) {
        self.bag.push((self.map1)(value));
    }

    fn fc(&self, values: impl ExactSizeIterator<Item = Self::Item>) {
        let values = values.map(&self.map1);
        self.bag.extend(values);
    }
}
