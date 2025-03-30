use super::xfx::Xfx;
use crate::computations::Values;
use crate::runner::{ComputationKind, ParallelRunner, ParallelRunnerCompute};
use orx_concurrent_iter::ConcurrentIter;

impl<I, Vt, Vo, M1, F, M2> Xfx<I, Vt, Vo, M1, F, M2>
where
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&Vt::Item) -> bool + Send + Sync,
    M2: Fn(Vt::Item) -> Vo + Send + Sync,
{
    pub fn reduce<R, Red>(self, reduce: Red) -> (usize, Option<Vo::Item>)
    where
        R: ParallelRunner,
        Red: Fn(Vo::Item, Vo::Item) -> Vo::Item + Send + Sync,
    {
        XfxReduce::compute::<R>(self, reduce)
    }
}

pub struct XfxReduce<I, Vt, Vo, M1, F, M2, X>
where
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&Vt::Item) -> bool + Send + Sync,
    M2: Fn(Vt::Item) -> Vo + Send + Sync,
    X: Fn(Vo::Item, Vo::Item) -> Vo::Item + Send + Sync,
{
    xfx: Xfx<I, Vt, Vo, M1, F, M2>,
    reduce: X,
}

impl<I, Vt, Vo, M1, F, M2, X> XfxReduce<I, Vt, Vo, M1, F, M2, X>
where
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&Vt::Item) -> bool + Send + Sync,
    M2: Fn(Vt::Item) -> Vo + Send + Sync,
    X: Fn(Vo::Item, Vo::Item) -> Vo::Item + Send + Sync,
{
    pub fn compute<R: ParallelRunner>(
        xfx: Xfx<I, Vt, Vo, M1, F, M2>,
        reduce: X,
    ) -> (usize, Option<Vo::Item>) {
        let xfx = Self { xfx, reduce };
        let p = xfx.xfx.params();
        match p.is_sequential() {
            true => (0, xfx.sequential()),
            false => xfx.parallel::<R>(),
        }
    }

    fn sequential(self) -> Option<Vo::Item> {
        let (xfx, reduce) = (self.xfx, self.reduce);
        let (_, iter, xap1, filter, xap2) = xfx.destruct();

        iter.into_seq_iter()
            .filter_map(|x| xap1(x).fx_reduce(None, &filter, &xap2, &reduce))
            .reduce(&reduce)
    }

    fn parallel<R: ParallelRunner>(self) -> (usize, Option<Vo::Item>) {
        let (xfx, reduce) = (self.xfx, self.reduce);
        let (params, iter, xap1, filter, xap2) = xfx.destruct();

        let runner = R::new(ComputationKind::Reduce, params, iter.try_get_len());
        runner.xfx_reduce(&iter, &xap1, &filter, &xap2, &reduce)
    }
}
