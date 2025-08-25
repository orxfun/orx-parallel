use crate::computational_variants::ParXap;
use crate::computations::X;
use crate::par_iter_fallible::IntoResult;
use crate::par_iter_optional::{IntoOption, ParIterOptional};
use crate::runner::{DefaultRunner, ParallelRunner};
use crate::values::TransformableValues;
use crate::values::runner_results::Infallible;
use crate::{IterationOrder, ParCollectInto, ParIter};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

pub struct ParXapOptional<I, T, Vo, M1, R = DefaultRunner>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    Vo::Item: IntoOption<T>,
    M1: Fn(I::Item) -> Vo + Sync,
{
    par: ParXap<I, Vo, M1, R>,
    phantom: PhantomData<T>,
}

impl<I, T, Vo, M1, R> ParXapOptional<I, T, Vo, M1, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    Vo::Item: IntoOption<T>,
    M1: Fn(I::Item) -> Vo + Sync,
{
    pub(crate) fn new(par: ParXap<I, Vo, M1, R>) -> Self {
        Self {
            par,
            phantom: PhantomData,
        }
    }
}

impl<I, T, Vo, M1, R> ParIterOptional<R> for ParXapOptional<I, T, Vo, M1, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    Vo::Item: IntoOption<T>,
    M1: Fn(I::Item) -> Vo + Sync,
    T: Send,
    Vo::Item: Send,
{
    type Success = T;

    type RegularItem = Vo::Item;

    type RegularParIter = ParXap<I, Vo, M1, R>;

    fn con_iter_len(&self) -> Option<usize> {
        self.par.con_iter().try_get_len()
    }

    fn into_regular_par(self) -> Self::RegularParIter {
        self.par
    }

    fn from_regular_par(regular_par: Self::RegularParIter) -> Self {
        Self {
            par: regular_par,
            phantom: PhantomData,
        }
    }

    // params transformations

    fn with_runner<Q: ParallelRunner>(self) -> impl ParIterOptional<Q, Success = Self::Success> {
        let (params, iter, m1) = self.par.destruct();
        ParXapOptional {
            par: ParXap::new(params, iter, m1),
            phantom: PhantomData,
        }
    }

    // // collect

    // fn collect_into<C>(self, output: C) -> Result<C, Self::Error>
    // where
    //     C: ParCollectInto<Self::Success>,
    // {
    //     let (params, iter, x1) = self.par.destruct();
    //     let x1 = |i: I::Item| x1(i).map_while_ok(|x| x.into_result());
    //     let x = X::new(params, iter, x1);
    //     output.x_try_collect_into::<R, _, _, _>(x)
    // }

    // // reduce

    // fn reduce<Reduce>(self, reduce: Reduce) -> Result<Option<Self::Success>, Self::Error>
    // where
    //     Self::Success: Send,
    //     Reduce: Fn(Self::Success, Self::Success) -> Self::Success + Sync,
    // {
    //     let (params, iter, x1) = self.par.destruct();
    //     let x1 = |i: I::Item| x1(i).map_while_ok(|x| x.into_result());
    //     let x = X::new(params, iter, x1);
    //     x.try_reduce::<R, _>(reduce).1
    // }

    // // early exit

    // fn first(self) -> Result<Option<Self::Success>, Self::Error>
    // where
    //     Self::Success: Send,
    // {
    //     let (params, iter, x1) = self.par.destruct();
    //     let x1 = |i: I::Item| x1(i).map_while_ok(|x| x.into_result());
    //     let x = X::new(params, iter, x1);
    //     match params.iteration_order {
    //         IterationOrder::Ordered => x.try_next::<R>().1,
    //         IterationOrder::Arbitrary => x.try_next_any::<R>().1,
    //     }
    // }
}
