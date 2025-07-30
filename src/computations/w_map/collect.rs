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

impl<I, T, O, M1> M<I, T, O, M1>
where
    I: ConcurrentIter,
    T: Send + Clone,
    O: Send + Sync,
    M1: Fn(&mut T, I::Item) -> O + Send + Sync,
{
}

pub struct MCollect<I, T, O, M1, P>
where
    I: ConcurrentIter,
    T: Send + Clone,
    O: Send + Sync,
    M1: Fn(&mut T, I::Item) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    m: M<I, T, O, M1>,
    pinned_vec: P,
}

impl<I, T, O, M1, P> MCollect<I, T, O, M1, P>
where
    I: ConcurrentIter,
    T: Send + Clone,
    O: Send + Sync,
    M1: Fn(&mut T, I::Item) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    fn sequential(self) -> P {
        let (m, mut pinned_vec) = (self.m, self.pinned_vec);
        let (_, iter, mut cmv, map1) = m.destruct();

        let iter = iter.into_seq_iter();
        for i in iter {
            pinned_vec.push(map1(&mut cmv, i));
        }

        pinned_vec
    }
}
