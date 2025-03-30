use super::m::M;
use crate::{
    computations::{map_self_atom, Atom},
    runner::{ComputationKind, ParallelRunner, ParallelRunnerCompute, ParallelTaskWithIdx},
};
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_pinned_vec::IntoConcurrentPinnedVec;
use std::marker::PhantomData;

pub struct MReduce<I, O, M1, X>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(I::Item) -> O + Send + Sync,
    X: Fn(O, O) -> O + Send + Sync,
{
    m: M<I, O, M1>,
    reduce: X,
}

impl<I, O, M1, X> MReduce<I, O, M1, X>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(I::Item) -> O + Send + Sync,
    X: Fn(O, O) -> O + Send + Sync,
{
    pub fn compute<R: ParallelRunner>(m: M<I, O, M1>, reduce: X) -> (usize, Option<O>) {
        let x = Self { m, reduce };
        let p = x.m.params();
        match p.is_sequential() {
            true => (0, x.sequential()),
            false => x.parallel::<R>(),
        }
    }

    fn sequential(self) -> Option<O> {
        let (m, reduce) = (self.m, self.reduce);
        let (_, iter, map1) = m.destruct();
        iter.into_seq_iter().map(map1).reduce(reduce)
    }

    fn parallel<R: ParallelRunner>(self) -> (usize, Option<O>) {
        let (m, reduce) = (self.m, self.reduce);
        let (params, iter, map1) = m.destruct();

        let runner = R::new(ComputationKind::Reduce, params, iter.try_get_len());
        let xap1 = |i: I::Item| Atom(map1(i));
        runner.x_reduce(&iter, &xap1, &reduce)
    }
}
