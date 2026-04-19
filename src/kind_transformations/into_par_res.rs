use crate::infallible::ParIter;
use crate::infallible::Xap;
use crate::infallible::xap_variants::Id;
use crate::result::{ParRes, SizePairRes};
use crate::runner::ParRunner;
use crate::sizes::IntoSizePair;
use orx_concurrent_iter::ConcurrentIter;

impl<O, E, I, X, R> ParIter<I, X, R>
where
    I: ConcurrentIter,
    X: Xap<I = I::Item, O = Result<O, E>>,
    X::Size: IntoSizePair,
    <X::Size as IntoSizePair>::ThenOne: SizePairRes,
    R: ParRunner,
{
    pub fn fallible_result(
        self,
    ) -> ParRes<I, O, E, X, Id<O>, <X::Size as IntoSizePair>::ThenOne, R> {
        let (iter, xap, exe, params) = self.destruct();
        ParRes::new(iter, xap, Id::new(), exe, params)
    }
}
