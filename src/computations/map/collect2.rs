use std::marker::PhantomData;

use super::m2::M;
use crate::{
    computations::Values, runner::ParallelTaskWithIdx, ChunkSize, CollectOrdering, NumThreads,
    Params,
};
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;

pub struct MCollect<I, Vo, M1, P>
where
    I: ConcurrentIter,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vo + Send + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    m: M<I, Vo, M1>,
    pinned_vec: P,
}

impl<I, Vo, M1, P> MCollect<I, Vo, M1, P>
where
    I: ConcurrentIter,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vo + Send + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    fn sequential(self) -> P {
        let (m, mut pinned_vec) = (self.m, self.pinned_vec);
        let (_, iter, map1) = m.destruct();

        let iter = iter.into_seq_iter();
        for i in iter {
            map1(i).push_to_pinned_vec(&mut pinned_vec);
        }

        pinned_vec
    }
}

// ordered

struct MCollectInInputOrder<'a, I, Vo, M1, P>
where
    I: Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I) -> Vo + Send + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    offset: usize,
    o_bag: &'a ConcurrentOrderedBag<Vo::Item, P>,
    map1: M1,
    phantom: PhantomData<I>,
}

impl<'a, I, Vo, M1, P> MCollectInInputOrder<'a, I, Vo, M1, P>
where
    I: Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I) -> Vo + Send + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
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

impl<'a, I, Vo, M1, P> ParallelTaskWithIdx for MCollectInInputOrder<'a, I, Vo, M1, P>
where
    I: Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I) -> Vo + Send + Sync,
    P: IntoConcurrentPinnedVec<Vo::Item>,
{
    type Item = I;

    fn f1(&self, idx: usize, value: Self::Item) {
        (self.map1)(value).push_to_ordered_bag(self.offset + idx, self.o_bag);
    }

    fn fc(&self, begin_idx: usize, values: impl ExactSizeIterator<Item = Self::Item>) {
        let n = values.len();
        let values = values.flat_map(|x| (self.map1)(x).values());
        let ok: Vec<Vo::Item> = values.collect();
        // values.push_to_ordered_bag_in_input_order(self.offset + begin_idx, n, self.o_bag);
    }
}
