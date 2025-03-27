use super::m::M;
use crate::runner::{ComputationKind, ParallelRunner, ParallelTaskWithIdx};
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_pinned_vec::IntoConcurrentPinnedVec;
use std::marker::PhantomData;

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
        match x.m.params().is_sequential() {
            true => (0, x.sequential()),
            false => x.parallel_in_input_order::<R>(),
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
        let offset = pinned_vec.len();
        let (params, iter, map1) = m.destruct();

        let bag: ConcurrentOrderedBag<O, P> = pinned_vec.into();
        let task = MCollectInInputOrder::new(offset, &bag, map1);

        let runner = R::new(ComputationKind::Collect, params, iter.try_get_len());
        let num_spawned = runner.new_run_idx(&iter, task);

        let values = unsafe { bag.into_inner().unwrap_only_if_counts_match() };
        (num_spawned, values)
    }
}

// collect in order

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

impl<'a, I, O, M1, P> ParallelTaskWithIdx for MCollectInInputOrder<'a, I, O, M1, P>
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
