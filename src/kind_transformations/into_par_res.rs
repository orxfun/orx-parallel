#![allow(refining_impl_trait)]

use crate::infallible::{Par, Xap, xap_variants::Id};
use crate::result::{ParRes, SizePairRes};
use crate::runner::ParRunner;
use crate::sizes::Size;
use crate::{ParIter, ParResIter};
use orx_concurrent_iter::ConcurrentIter;

pub trait IntoParResIter: ParIter<Item = Result<Self::Success, Self::Error>> {
    type Success;

    type Error;

    fn fallible_result(
        self,
    ) -> impl ParResIter<
        Runner = Self::Runner,
        Size = <Self::Size as Size>::ThenOne,
        Item = Self::Success,
        Error = Self::Error,
    >;
}

impl<O, E, I, X, R> IntoParResIter for Par<I, X, R>
where
    I: ConcurrentIter,
    X: Xap<I = I::Item, O = Result<O, E>>,
    <X::Size as Size>::ThenOne: SizePairRes,
    R: ParRunner,
{
    type Success = O;

    type Error = E;

    fn fallible_result(self) -> ParRes<I, O, E, X, Id<O>, <X::Size as Size>::ThenOne, R> {
        let (iter, xap, exe, params) = self.destruct();
        ParRes::new(iter, xap, Id::new(), exe, params)
    }
}
