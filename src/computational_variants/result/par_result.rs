use crate::computations::X;
use crate::par_iter_result::ParIterResult;
use crate::runner::{DefaultRunner, ParallelRunner};
use crate::values::WhilstOk;
use crate::{ParCollectInto, Params};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

/// A parallel iterator.
pub struct ParResult<I, T, E, R = DefaultRunner>
where
    R: ParallelRunner,
    I: ConcurrentIter<Item = Result<T, E>>,
{
    iter: I,
    params: Params,
    phantom: PhantomData<R>,
}

impl<I, T, E, R> ParResult<I, T, E, R>
where
    R: ParallelRunner,
    I: ConcurrentIter<Item = Result<T, E>>,
{
    fn destruct(self) -> (Params, I) {
        (self.params, self.iter)
    }
}

impl<I, T, E, R> ParIterResult for ParResult<I, T, E, R>
where
    R: ParallelRunner,
    I: ConcurrentIter<Item = Result<T, E>>,
{
    type Item = T;

    type Error = E;

    fn con_iter_len(&self) -> Option<usize> {
        self.iter.try_get_len()
    }

    fn collect_into<C>(self, output: C) -> Result<C, Self::Error>
    where
        C: ParCollectInto<Self::Item>,
        E: Send,
    {
        let (params, iter) = self.destruct();
        let x1 = move |i: I::Item| WhilstOk::<T, E>::new(i);
        let x = X::new(params, iter, x1);
        output.x_try_collect_into::<R, _, _, _>(x)
    }
}
