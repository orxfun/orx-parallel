use super::m::M;
use crate::computations::Atom;
use crate::runner::{ComputationKind, ParallelRunner, ParallelRunnerCompute};
use orx_concurrent_iter::ConcurrentIter;

impl<I, O, M1> M<I, O, M1>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(I::Item) -> O + Send + Sync,
{
    pub fn reduce<R, X>(self, reduce: X) -> (usize, Option<O>)
    where
        R: ParallelRunner,
        X: Fn(O, O) -> O + Send + Sync,
    {
        let p = self.params();
        match p.is_sequential() {
            true => (0, self.reduce_sequential(reduce)),
            false => self.reduce_parallel::<R, _>(reduce),
        }
    }

    fn reduce_sequential<X>(self, reduce: X) -> Option<O>
    where
        X: Fn(O, O) -> O + Send + Sync,
    {
        let (_, iter, map1) = self.destruct();
        iter.into_seq_iter().map(map1).reduce(reduce)
    }

    fn reduce_parallel<R, X>(self, reduce: X) -> (usize, Option<O>)
    where
        R: ParallelRunner,
        X: Fn(O, O) -> O + Send + Sync,
    {
        let (params, iter, map1) = self.destruct();

        let runner = R::new(ComputationKind::Reduce, params, iter.try_get_len());
        let xap1 = |i: I::Item| Atom(map1(i));
        let create_map = || xap1.clone();
        runner.x_reduce(&iter, create_map, &reduce)
    }
}
