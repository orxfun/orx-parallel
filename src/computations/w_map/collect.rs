use super::m::WithM;
use crate::runner::{ComputationKind, ParallelRunner, ParallelRunnerCompute, ParallelTaskWithIdx};
#[cfg(test)]
use crate::{IterationOrder, runner::ParallelTask};
#[cfg(test)]
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_pinned_vec::IntoConcurrentPinnedVec;
use std::marker::PhantomData;

impl<I, T, O, M1> WithM<I, T, O, M1>
where
    I: ConcurrentIter,
    T: Send + Clone,
    O: Send + Sync,
    M1: Fn(&mut T, I::Item) -> O + Send + Sync,
{
}

pub struct WithMCollect<I, T, O, M1, P>
where
    I: ConcurrentIter,
    T: Send + Clone,
    O: Send + Sync,
    M1: Fn(&mut T, I::Item) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    m: WithM<I, T, O, M1>,
    pinned_vec: P,
}

impl<I, T, O, M1, P> WithMCollect<I, T, O, M1, P>
where
    I: ConcurrentIter,
    T: Send + Clone,
    O: Send + Sync,
    M1: Fn(&mut T, I::Item) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    fn sequential(self) -> P {
        let (m, mut pinned_vec) = (self.m, self.pinned_vec);
        let (_, iter, mut with, map1) = m.destruct();

        let iter = iter.into_seq_iter();
        for i in iter {
            pinned_vec.push(map1(&mut with, i));
        }

        pinned_vec
    }

    // fn parallel_in_input_order<R: ParallelRunner>(self) -> (usize, P) {
    //     let (m, pinned_vec) = (self.m, self.pinned_vec);
    //     let offset = pinned_vec.len();
    //     let (params, iter, with, map1) = m.destruct();

    //     let bag: ConcurrentOrderedBag<O, P> = pinned_vec.into();
    //     let task = MCollectInInputOrder::new(offset, &bag, with, map1);

    //     let runner = R::new(ComputationKind::Collect, params, iter.try_get_len());
    //     let num_spawned = runner.run_with_idx(&iter, task);

    //     let values = unsafe { bag.into_inner().unwrap_only_if_counts_match() };
    //     (num_spawned, values)
    // }
}

// ordered

struct MCollectInInputOrder<'a, I, T, O, M1, P>
where
    T: Send + Clone,
    O: Send + Sync,
    M1: Fn(&mut T, I) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    offset: usize,
    o_bag: &'a ConcurrentOrderedBag<O, P>,
    with: T,
    map1: &'a M1,
    phantom: PhantomData<I>,
}

impl<'a, I, T, O, M1, P> MCollectInInputOrder<'a, I, T, O, M1, P>
where
    T: Send + Clone,
    O: Send + Sync,
    M1: Fn(&mut T, I) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    fn new(offset: usize, o_bag: &'a ConcurrentOrderedBag<O, P>, with: T, map1: &'a M1) -> Self {
        Self {
            offset,
            o_bag,
            with,
            map1,
            phantom: PhantomData,
        }
    }
}

impl<'a, I, T, O, M1, P> Clone for MCollectInInputOrder<'a, I, T, O, M1, P>
where
    T: Send + Clone,
    O: Send + Sync,
    M1: Fn(&mut T, I) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    fn clone(&self) -> Self {
        Self {
            offset: self.offset,
            o_bag: self.o_bag,
            with: self.with.clone(),
            map1: self.map1,
            phantom: self.phantom,
        }
    }
}

impl<'a, I, T, O, M1, P> ParallelTaskWithIdx for MCollectInInputOrder<'a, I, T, O, M1, P>
where
    T: Send + Clone,
    O: Send + Sync,
    M1: Fn(&mut T, I) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    type Item = I;

    fn f1(&mut self, idx: usize, value: Self::Item) {
        // unsafe {
        //     self.o_bag
        //         .set_value(self.offset + idx, (self.map1)(&mut self.with, value))
        // };
    }

    fn fc(&mut self, begin_idx: usize, values: impl ExactSizeIterator<Item = Self::Item>) {
        todo!()
        // let values = values.map(&self.map1);
        // unsafe { self.o_bag.set_values(self.offset + begin_idx, values) };
    }
}
