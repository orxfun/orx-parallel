use crate::computations::X;
use crate::par_iter_result::ParIterResult;
use crate::runner::{DefaultRunner, ParallelRunner};
use crate::values::{TransformableValues, WhilstOk, WhilstOkVector};
use crate::{ParCollectInto, Params};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

/// A parallel iterator.
pub struct ParXapResult<I, Vo, M1, T, E, Mr, R = DefaultRunner>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: TransformableValues,
    M1: Fn(I::Item) -> Vo + Sync,
    Mr: Fn(Vo::Item) -> Result<T, E> + Sync + Clone, // TODO: check this clone
{
    iter: I,
    params: Params,
    xap1: M1,
    map_res: Mr,
    phantom: PhantomData<R>,
}

impl<I, Vo, M1, T, E, Mr, R> ParXapResult<I, Vo, M1, T, E, Mr, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: TransformableValues,
    M1: Fn(I::Item) -> Vo + Sync,
    Mr: Fn(Vo::Item) -> Result<T, E> + Sync + Clone,
{
    pub(crate) fn new(iter: I, params: Params, xap1: M1, map_res: Mr) -> Self {
        Self {
            iter,
            params,
            xap1,
            map_res,
            phantom: PhantomData,
        }
    }

    fn destruct(self) -> (Params, I, M1, Mr) {
        (self.params, self.iter, self.xap1, self.map_res)
    }
}

impl<I, Vo, M1, T, E, Mr, R> ParIterResult<R> for ParXapResult<I, Vo, M1, T, E, Mr, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: TransformableValues,
    M1: Fn(I::Item) -> Vo + Sync,
    Mr: Fn(Vo::Item) -> Result<T, E> + Sync + Clone,
{
    type Item = T;

    type Error = E;

    fn con_iter_len(&self) -> Option<usize> {
        self.iter.try_get_len()
    }

    fn collect_into<C>(self, output: C) -> Result<C, Self::Error>
    where
        C: ParCollectInto<Self::Item>,
        Self::Error: Send,
    {
        let (params, iter, xap, mr) = self.destruct();
        let x1 = move |i: I::Item| {
            let v1: Vo = xap(i);
            let iter = v1.values().into_iter();
            let mr = mr.clone();
            let iter_result = iter.map(move |x| WhilstOk(mr(x)));
            WhilstOkVector(iter_result)
        };
        let x = X::new(params, iter, x1);
        output.x_try_collect_into::<R, _, _, _>(x)
    }
}
