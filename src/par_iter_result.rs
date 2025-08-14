use crate::{
    DefaultRunner, ParCollectInto, ParallelRunner,
    computations::X,
    values::{TransformableValues, WhilstOk},
};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

pub struct ParIterResult3<I, T, E, M1, R = DefaultRunner>
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

impl<I, T, E, M1, R> ParIterResult3<I, T, E, M1, R>
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

// try again

pub struct ParIterResult2<I, T, E, Vo, M1, R = DefaultRunner>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: TransformableValues<Item = T, Error = E>,
    M1: Fn(I::Item) -> Vo + Sync,
    T: Send + Sync,
    E: Send + Sync,
{
    x: X<I, Vo, M1>,
    con_iter_len: Option<usize>,
    phantom: PhantomData<(Vo, R)>,
}

impl<I, T, E, Vo, M1, R> ParIterResult2<I, T, E, Vo, M1, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: TransformableValues<Item = T, Error = E>,
    M1: Fn(I::Item) -> Vo + Sync,
    T: Send + Sync,
    E: Send + Sync,
{
    pub(crate) fn new(x: X<I, Vo, M1>, con_iter_len: Option<usize>) -> Self {
        Self {
            x,
            con_iter_len,
            phantom: PhantomData,
        }
    }

    pub fn collect_result_into<C>(self, output: C) -> Result<C, Vo::Error>
    where
        C: ParCollectInto<Vo::Item>,
    {
        output.x_try_collect_into::<R, _, _, _>(self.x)
    }

    pub fn collect<C>(self) -> Result<C, Vo::Error>
    where
        C: ParCollectInto<Vo::Item>,
    {
        let output = C::empty(self.con_iter_len);
        self.collect_result_into(output)
    }
}

// as a trait

pub trait ParIterResult<R = DefaultRunner>
where
    R: ParallelRunner,
{
    type Item;

    type Error;

    fn con_iter_len(&self) -> Option<usize>;

    fn collect_into<C>(self, output: C) -> Result<C, Self::Error>
    where
        C: ParCollectInto<Self::Item>,
        Self::Error: Send;

    fn collect<C>(self) -> Result<C, Self::Error>
    where
        C: ParCollectInto<Self::Item>,
        Self::Error: Send,
        Self: Sized,
    {
        let output = C::empty(self.con_iter_len());
        self.collect_into(output)
    }
}
