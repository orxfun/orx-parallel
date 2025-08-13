use std::marker::PhantomData;

use crate::{
    DefaultRunner, ParCollectInto, ParIter, ParallelRunner, computations::X, values::WhilstOk,
};
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_option::{ConcurrentOption, IntoOption};
use orx_fixed_vec::IntoConcurrentPinnedVec;

pub trait ParIterResult<T, E>: ParIter<Item = Result<T, E>> {
    fn collect_result_into<C>(self, output: C) -> Result<C, E>
    where
        C: ParCollectInto<T>,
        E: Send + Sync,
    {
        let error = ConcurrentOption::<E>::none();
        let result = self
            .map(|x| match x {
                Ok(x) => Some(x),
                Err(e) => {
                    _ = error.set_some(e);
                    None
                }
            })
            .take_while(|x| x.is_some())
            .map(|x| {
                // SAFETY: since x passed the whilst(is-some) check, unwrap_unchecked
                unsafe { x.unwrap_unchecked() }
            })
            .collect_into(output);

        match error.into_option() {
            None => Ok(result),
            Some(e) => Err(e),
        }
    }

    fn collect_result<C>(self) -> Result<C, E>
    where
        C: ParCollectInto<T>,
        E: Send + Sync,
    {
        let output = C::empty(self.con_iter().try_get_len());
        self.collect_result_into(output)
    }
}

impl<P, T, E> ParIterResult<T, E> for P where P: ParIter<Item = Result<T, E>> {}

// TODO NEW ONE

pub trait ParIterResultNew<T, E, R = DefaultRunner>: ParIter<R, Item = Result<T, E>>
where
    T: Send,
    E: Send + Sync,
    R: ParallelRunner,
{
    fn collect_result_into_new<C>(self, output: C) -> Result<C, E>
    where
        C: ParCollectInto<T>,
        E: Send + Sync;

    fn collect_result_new<C>(self) -> Result<C, E>
    where
        C: ParCollectInto<T>,
        E: Send + Sync,
    {
        let output = C::empty(self.con_iter().try_get_len());
        self.collect_result_into_new(output)
    }
}

// AS STRUCT

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

    pub fn collect_result<C>(self) -> Result<C, E>
    where
        C: ParCollectInto<T>,
        E: Send + Sync,
    {
        let output = C::empty(self.con_iter_len);
        self.collect_result_into(output)
    }
}
