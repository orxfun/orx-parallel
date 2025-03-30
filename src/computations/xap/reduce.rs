use super::x::X;
use crate::computations::{Atom, Values};
use crate::runner::{ComputationKind, ParallelRunner, ParallelRunnerCompute};
use orx_concurrent_iter::ConcurrentIter;

pub struct XReduce<I, Vo, M1, Red>
where
    I: ConcurrentIter,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vo + Send + Sync,
    Red: Fn(Vo::Item, Vo::Item) -> Vo::Item + Send + Sync,
{
    x: X<I, Vo, M1>,
    reduce: Red,
}

impl<I, Vo, M1, Red> XReduce<I, Vo, M1, Red>
where
    I: ConcurrentIter,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vo + Send + Sync,
    Red: Fn(Vo::Item, Vo::Item) -> Vo::Item + Send + Sync,
{
    pub fn compute<R: ParallelRunner>(x: X<I, Vo, M1>, reduce: Red) -> (usize, Option<Vo::Item>) {
        let x = Self { x, reduce };
        let p = x.x.params();
        match p.is_sequential() {
            true => (0, x.sequential()),
            false => x.parallel::<R>(),
        }
    }

    fn sequential(self) -> Option<Vo::Item> {
        let (x, reduce) = (self.x, self.reduce);
        let (_, iter, xap1) = x.destruct();
        iter.into_seq_iter()
            .filter_map(|x| xap1(x).acc_reduce(None, &reduce))
            .reduce(&reduce)
    }

    fn parallel<R: ParallelRunner>(self) -> (usize, Option<Vo::Item>) {
        let (x, reduce) = (self.x, self.reduce);
        let (params, iter, xap1) = x.destruct();

        let runner = R::new(ComputationKind::Reduce, params, iter.try_get_len());
        runner.x_reduce(&iter, &xap1, &reduce)
    }
}
