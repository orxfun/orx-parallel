#![allow(refining_impl_trait)]

use crate::infallible::{Par, Xap, xap_variants::Id};
use crate::result::{ParRes, SizePairRes};
use crate::runner::ParRunner;
use crate::sizes::IntoSizePair;
use crate::{ParIter, ParResIter};
use orx_concurrent_iter::ConcurrentIter;

pub trait IntoParResIter: ParIter<Item = Result<Self::Success, Self::Error>> {
    type Success;

    type Error;

    fn fallible_option(
        self,
    ) -> impl ParResIter<
        Runner = Self::Runner,
        Size = <Self::Size as IntoSizePair>::ThenOne,
        Item = Self::Success,
        Error = Self::Error,
    >
    where
        Self::Size: IntoSizePair;
}

impl<O, E, I, X, R> IntoParResIter for Par<I, X, R>
where
    I: ConcurrentIter,
    X: Xap<I = I::Item, O = Result<O, E>>,
    X::Size: IntoSizePair,
    <X::Size as IntoSizePair>::ThenOne: SizePairRes,
    R: ParRunner,
{
    type Success = O;

    type Error = E;

    fn fallible_option(self) -> ParRes<I, O, E, X, Id<O>, <X::Size as IntoSizePair>::ThenOne, R>
    where
        Self::Size: IntoSizePair,
    {
        let (iter, xap, exe, params) = self.destruct();
        ParRes::new(iter, xap, Id::new(), exe, params)
    }
}
