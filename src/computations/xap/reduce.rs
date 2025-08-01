use super::x::X;
use crate::computations::Values;
use crate::runner::{ComputationKind, ParallelRunner, ParallelRunnerCompute};
use orx_concurrent_iter::ConcurrentIter;

impl<I, Vo, M1> X<I, Vo, M1>
where
    I: ConcurrentIter,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vo + Send + Sync + Clone,
{
    pub fn reduce<R, Red>(self, reduce: Red) -> (usize, Option<Vo::Item>)
    where
        R: ParallelRunner,
        Red: Fn(Vo::Item, Vo::Item) -> Vo::Item + Send + Sync,
    {
        let x = XReduce { x: self, reduce };
        let p = x.x.params();
        match p.is_sequential() {
            true => (0, x.sequential()),
            false => x.parallel::<R>(),
        }
    }
}

pub struct XReduce<I, Vo, M1, Red>
where
    I: ConcurrentIter,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vo + Send + Sync + Clone,
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
    M1: Fn(I::Item) -> Vo + Send + Sync + Clone,
    Red: Fn(Vo::Item, Vo::Item) -> Vo::Item + Send + Sync,
{
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
        let create_map = || xap1.clone();
        runner.x_reduce(&iter, create_map, &reduce)
    }
}
