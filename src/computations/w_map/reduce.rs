use super::m::WithM;
use crate::computations::Atom;
use crate::runner::{ComputationKind, ParallelRunner, ParallelRunnerCompute};
use orx_concurrent_iter::ConcurrentIter;

impl<I, T, O, M1> WithM<I, T, O, M1>
where
    I: ConcurrentIter,
    T: Send + Clone,
    O: Send + Sync,
    M1: Fn(&mut T, I::Item) -> O + Send + Sync,
{
    fn reduce_sequential<X>(self, reduce: X) -> Option<O>
    where
        X: Fn(O, O) -> O + Send + Sync,
    {
        let (_, iter, mut with, map1) = self.destruct();
        iter.into_seq_iter()
            .map(|value| map1(&mut with, value))
            .reduce(reduce)
    }

    // fn reduce_parallel<R: ParallelRunner, X>(self, reduce: X) -> (usize, Option<O>)
    // where
    //     R: ParallelRunner,
    //     X: Fn(O, O) -> O + Send + Sync,
    // {
    //     let (params, iter, mut with, map1) = self.destruct();

    //     let runner = R::new(ComputationKind::Reduce, params, iter.try_get_len());
    //     let xap1 = |i: I::Item| Atom(map1(&mut with, i));
    //     runner.x_reduce(&iter, &xap1, &reduce)
    // }
}
