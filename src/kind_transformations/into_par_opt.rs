#![allow(refining_impl_trait)]

use crate::infallible::{Par, Xap, xap_variants::Id};
use crate::option::{ParOpt, SizePairOpt};
use crate::runner::ParRunner;
use crate::sizes::IntoSizePair;
use crate::{ParIter, ParOptIter};
use orx_concurrent_iter::ConcurrentIter;

pub trait IntoParOptIter: ParIter<Item = Option<Self::Success>> {
    type Success;

    fn fallible_option(
        self,
    ) -> impl ParOptIter<
        Runner = Self::Runner,
        Size = <Self::Size as IntoSizePair>::ThenOne,
        Item = Self::Success,
    >
    where
        Self::Size: IntoSizePair;
}

impl<O, I, X, R> IntoParOptIter for Par<I, X, R>
where
    I: ConcurrentIter,
    X: Xap<I = I::Item, O = Option<O>>,
    X::Size: IntoSizePair,
    <X::Size as IntoSizePair>::ThenOne: SizePairOpt,
    R: ParRunner,
{
    type Success = O;

    fn fallible_option(self) -> ParOpt<I, O, X, Id<O>, <X::Size as IntoSizePair>::ThenOne, R>
    where
        Self::Size: IntoSizePair,
    {
        let (iter, xap, exe, params) = self.destruct();
        ParOpt::new(iter, xap, Id::new(), exe, params)
    }
}
