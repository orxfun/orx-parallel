#![allow(refining_impl_trait)]

use crate::infallible::{Par, ParIterDestruct, Xap, xap_variants::Id};
use crate::option::ParOpt;
use crate::runner::ParRunner;
use crate::sizes::Size;
use crate::{ParIter, ParOptIter};
use orx_concurrent_iter::ConcurrentIter;

pub trait IntoParOptIter: ParIter<Item = Option<Self::Success>> {
    type Success;

    fn fallible_option(
        self,
    ) -> impl ParOptIter<
        Runner = Self::Runner,
        // Size = <<Self::Xap as Xap>::Size as Size>::IntoPair,
        Item = Self::Success,
    >;
}

impl<O, I, X, R> IntoParOptIter for Par<I, X, R>
where
    I: ConcurrentIter,
    X: Xap<I = I::Item, O = Option<O>>,
    R: ParRunner,
{
    type Success = O;

    fn fallible_option(self) -> ParOpt<I, O, X, Id<O>, <X::Size as Size>::IntoPair, R> {
        let (iter, xap, exe, params) = self.destruct();
        ParOpt::new(iter, xap, Id::new(), exe, params)
    }
}

// fn to_fallible_opt<P: ParIter<Item = Option<usize>>>(
//     par: P,
// ) -> impl ParOptIter<Runner = P::Runner, Size = <<P::Xap as Xap>::Size as Size>::IntoPair, Item = usize>
// {
//     let (iter, xap, exe, params) = par.destructor();
//     ParOpt::new(iter, xap, Id::new(), exe, params)
// }

fn abc() {
    use super::*;
    use crate::*;

    fn get_par(n: usize) -> impl ParIter<Item = Option<usize>> {
        (0..n).par().map(|x| x + 1).map(Some)
    }

    // let par = (0..10).par().map(|x| x + 1).map(Some);
    let par = get_par(10);
    // let par = par.fallible_option2();
    // let par = to_fallible_opt(par);
}
