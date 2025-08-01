use super::x::WithX;
use crate::computations::Values;
use crate::runner::{ComputationKind, ParallelRunner, ParallelRunnerCompute};
use orx_concurrent_iter::ConcurrentIter;

impl<I, T, Vo, M1> WithX<I, T, Vo, M1>
where
    I: ConcurrentIter,
    T: Send + Sync + Clone,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(&mut T, I::Item) -> Vo + Clone + Send + Sync,
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

pub struct XReduce<I, T, Vo, M1, Red>
where
    I: ConcurrentIter,
    T: Send + Sync + Clone,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(&mut T, I::Item) -> Vo + Clone + Send + Sync,
    Red: Fn(Vo::Item, Vo::Item) -> Vo::Item + Send + Sync,
{
    x: WithX<I, T, Vo, M1>,
    reduce: Red,
}

impl<I, T, Vo, M1, Red> XReduce<I, T, Vo, M1, Red>
where
    I: ConcurrentIter,
    T: Send + Sync + Clone,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(&mut T, I::Item) -> Vo + Clone + Send + Sync,
    Red: Fn(Vo::Item, Vo::Item) -> Vo::Item + Send + Sync,
{
    fn sequential(self) -> Option<Vo::Item> {
        let (x, reduce) = (self.x, self.reduce);
        let (_, iter, mut with, xap1) = x.destruct();
        iter.into_seq_iter()
            .filter_map(|x| xap1(&mut with, x).acc_reduce(None, &reduce))
            .reduce(&reduce)
    }

    fn parallel<R: ParallelRunner>(self) -> (usize, Option<Vo::Item>) {
        let (x, reduce) = (self.x, self.reduce);
        let (params, iter, with, xap1) = x.destruct();

        let runner = R::new(ComputationKind::Reduce, params, iter.try_get_len());
        let create_map = || {
            let mut with = with.clone();
            let xap1 = xap1.clone();
            move |value| xap1(&mut with, value)
        };
        runner.x_reduce(&iter, create_map, &reduce)
    }
}
