use super::m::WithM;
use crate::computations::Atom;
use crate::runner::{ComputationKind, ParallelRunner, ParallelRunnerCompute};
use orx_concurrent_iter::ConcurrentIter;

impl<I, T, O, M1> WithM<I, T, O, M1>
where
    I: ConcurrentIter,
    T: Send + Sync + Clone,
    O: Send + Sync,
    M1: Fn(&mut T, I::Item) -> O + Clone + Send + Sync,
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
        let (_, iter, mut with, map1) = self.destruct();
        iter.into_seq_iter()
            .map(|value| map1(&mut with, value))
            .reduce(reduce)
    }

    fn reduce_parallel<R: ParallelRunner, X>(self, reduce: X) -> (usize, Option<O>)
    where
        R: ParallelRunner,
        X: Fn(O, O) -> O + Send + Sync,
    {
        let (params, iter, mut with, map1) = self.destruct();

        let runner = R::new(ComputationKind::Reduce, params, iter.try_get_len());
        let create_map = || {
            let map1 = map1.clone();
            let mut with = with.clone();
            move |i: I::Item| Atom(map1(&mut with, i))
        };
        runner.x_reduce(&iter, create_map, &reduce)
    }
}
