use crate::computational_variants::Par;
use crate::default_fns::map_self;
use crate::executor::parallel_compute as prc;
use crate::par_iter_result::{IntoResult, ParIterResult};
use crate::runner::{DefaultRunner, ParallelRunner};
use crate::{IterationOrder, ParCollectInto, ParIter};
use core::marker::PhantomData;
use orx_concurrent_iter::ConcurrentIter;

/// A parallel iterator for which the computation either completely succeeds,
/// or fails and **early exits** with None.
pub struct ParOption<I, T, R = DefaultRunner>
where
    R: ParallelRunner,
    I: ConcurrentIter<Item = Option<T>>,
{
    par: Par<I, R>,
    phantom: PhantomData<T>,
}

impl<I, T, R> ParOption<I, T, R>
where
    R: ParallelRunner,
    I: ConcurrentIter<Item = Option<T>>,
{
    pub(crate) fn new(par: Par<I, R>) -> Self {
        Self {
            par,
            phantom: PhantomData,
        }
    }

    pub fn con_iter_len(&self) -> Option<usize> {
        self.par.con_iter().try_get_len()
    }

    pub fn into_regular_par(self) -> Par<I, R> {
        self.par
    }

    fn from_regular_par(regular_par: Par<I, R>) -> Self {
        Self {
            par: regular_par,
            phantom: PhantomData,
        }
    }

    // params transformations

    fn with_runner<Q: ParallelRunner>(self, orchestrator: Q) -> ParOption<I, T, Q> {
        let (_, params, iter) = self.par.destruct();
        ParOption::new(Par::new(orchestrator, params, iter))
    }

    // collect

    fn collect_into<C>(self, output: C) -> Option<C>
    where
        C: ParCollectInto<T>,
        T: Send,
    {
        let (orchestrator, params, iter) = self.par.destruct();
        output.x_maybe_collect_into(orchestrator, params, iter, map_self);
        // let x1 = |i: I::Item| i.into_result();
        // output.x_try_collect_into(orchestrator, params, iter, x1)
        None
    }
}
