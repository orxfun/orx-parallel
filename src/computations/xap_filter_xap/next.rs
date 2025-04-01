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
    pub fn next<R>(self) -> (usize, Option<Vo::Item>)
    where
        R: ParallelRunner,
    {
        XfxNext(self).compute::<R>()
    }
}

// next

struct XfxNext<I, Vt, Vo, M1, F, M2>(Xfx<I, Vt, Vo, M1, F, M2>)
where
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&Vt::Item) -> bool + Send + Sync,
    M2: Fn(Vt::Item) -> Vo + Send + Sync;

impl<I, Vt, Vo, M1, F, M2> XfxNext<I, Vt, Vo, M1, F, M2>
where
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&Vt::Item) -> bool + Send + Sync,
    M2: Fn(Vt::Item) -> Vo + Send + Sync,
{
    pub fn compute<R>(self) -> (usize, Option<Vo::Item>)
    where
        R: ParallelRunner,
    {
        match self.0.params().is_sequential() {
            true => (0, self.sequential()),
            false => self.parallel::<R>(),
        }
    }

    fn sequential(self) -> Option<Vo::Item> {
        let (_, iter, xap1, filter, xap2) = self.0.destruct();

        iter.into_seq_iter()
            .flat_map(|i| xap1(i).values().into_iter())
            .filter(|t| filter(t))
            .flat_map(|t| xap2(t).values().into_iter())
            .next()
    }

    fn parallel<R: ParallelRunner>(self) -> (usize, Option<Vo::Item>) {
        let (params, iter, xap1, filter, xap2) = self.0.destruct();
        let runner = R::new(ComputationKind::EarlyReturn, params, iter.try_get_len());
        runner.xfx_next(&iter, &xap1, &filter, &xap2)
    }
}

// next any

struct XfxNextAny<I, Vt, Vo, M1, F, M2>(Xfx<I, Vt, Vo, M1, F, M2>)
where
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&Vt::Item) -> bool + Send + Sync,
    M2: Fn(Vt::Item) -> Vo + Send + Sync;

impl<I, Vt, Vo, M1, F, M2> XfxNextAny<I, Vt, Vo, M1, F, M2>
where
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&Vt::Item) -> bool + Send + Sync,
    M2: Fn(Vt::Item) -> Vo + Send + Sync,
{
    pub fn compute<R>(self) -> (usize, Option<Vo::Item>)
    where
        R: ParallelRunner,
    {
        match self.0.params().is_sequential() {
            true => (0, self.sequential()),
            false => self.parallel::<R>(),
        }
    }

    fn sequential(self) -> Option<Vo::Item> {
        XfxNext(self.0).sequential()
    }

    fn parallel<R: ParallelRunner>(self) -> (usize, Option<Vo::Item>) {
        let (params, iter, xap1, filter, xap2) = self.0.destruct();
        let runner = R::new(ComputationKind::EarlyReturn, params, iter.try_get_len());
        runner.xfx_next(&iter, &xap1, &filter, &xap2)
    }
}
