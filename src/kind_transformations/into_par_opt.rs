#![allow(refining_impl_trait)]

use crate::infallible::{Par, Xap, xap_variants::Id};
use crate::option::{ParOpt, SizePairOpt};
use crate::runner::ParRunner;
use crate::sizes::Size;
use crate::{ParIter, ParOptIter};
use orx_concurrent_iter::ConcurrentIter;

pub trait IntoParOptIter: ParIter<Item = Option<Self::Success>> {
    type Success;

    fn fallible_option(
        self,
    ) -> impl ParOptIter<Runner = Self::Runner, Size = <Self::Size as Size>::ThenOne, Item = Self::Success>;
}

impl<O, I, X, R> IntoParOptIter for Par<I, X, R>
where
    I: ConcurrentIter,
    X: Xap<I = I::Item, O = Option<O>>,
    <X::Size as Size>::ThenOne: SizePairOpt,
    R: ParRunner,
{
    type Success = O;

    fn fallible_option(self) -> ParOpt<I, O, X, Id<O>, <X::Size as Size>::ThenOne, R> {
        let (iter, xap, exe, params) = self.destruct();
        ParOpt::new(iter, xap, Id::new(), exe, params)
    }
}

fn abc() {
    use super::*;
    use crate::*;

    fn get_par(n: usize) -> impl IntoParOptIter<Success = usize> {
        (0..n).par().map(|x| x + 1).map(Some)
    }

    // let par = (0..10).par().map(|x| x + 1).map(Some);
    let par = get_par(10);
    // let par = par.fallible_option();
}
