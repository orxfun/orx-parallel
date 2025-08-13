use crate::{DefaultRunner, ParCollectInto, ParallelRunner, computations::X, values::WhilstOk};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

pub struct ParIterResultStruct<I, T, E, M1, R = DefaultRunner>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    M1: Fn(I::Item) -> WhilstOk<T, E> + Sync,
    E: Send,
{
    x: X<I, WhilstOk<T, E>, M1>,
    con_iter_len: Option<usize>,
    phantom: PhantomData<R>,
}

impl<I, T, E, M1, R> ParIterResultStruct<I, T, E, M1, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    M1: Fn(I::Item) -> WhilstOk<T, E> + Sync,
    E: Send,
{
    pub(crate) fn new(x: X<I, WhilstOk<T, E>, M1>, con_iter_len: Option<usize>) -> Self {
        Self {
            x,
            con_iter_len,
            phantom: PhantomData,
        }
    }

    pub fn collect_result_into<C>(self, output: C) -> Result<C, E>
    where
        C: ParCollectInto<T>,
    {
        output.x_try_collect_into::<R, _, _, _>(self.x)
    }

    pub fn collect<C>(self) -> Result<C, E>
    where
        C: ParCollectInto<T>,
        E: Send + Sync,
    {
        let output = C::empty(self.con_iter_len);
        self.collect_result_into(output)
    }
}
