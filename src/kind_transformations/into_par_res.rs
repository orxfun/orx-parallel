#![allow(refining_impl_trait)]

use crate::infallible::{Par, ParIterCore, Xap, xap_variants::Id};
use crate::result::ParRes;
use crate::runner::ParRunner;
use crate::sizes::{Size, SizePair};
use crate::{ParIter, ParResIter};
use orx_concurrent_iter::ConcurrentIter;

pub trait IntoParResIter: ParIter<Item = Result<Self::Success, Self::Error>> {
    type Success;

    type Error;

    fn fallible_result(
        self,
    ) -> impl ParResIter<
        Runner = Self::Runner,
        // Size = <<Self::Xap as Xap>::Size as Size>::IntoPair,
        Item = Self::Success,
        Error = Self::Error,
    >;
}

impl<O, E, I, X, R> IntoParResIter for Par<I, X, R>
where
    I: ConcurrentIter,
    X: Xap<I = I::Item, O = Result<O, E>>,
    <X::Size as Size>::IntoPair: SizePair,
    R: ParRunner,
{
    type Success = O;

    type Error = E;

    fn fallible_result(self) -> ParRes<I, O, E, X, Id<O>, <X::Size as Size>::IntoPair, R> {
        let (iter, xap, exe, params) = self.destruct();
        ParRes::new(iter, xap, Id::new(), exe, params)
    }
}
