use crate::infallible::Par;
use crate::infallible::Xap;
use crate::infallible::xap_variants::Id;
use crate::result::ParRes;
use crate::result::XapRes;
use crate::result::size_pairs::IntoSizePair;
use crate::runner::ParRunner;
use orx_concurrent_iter::ConcurrentIter;

impl<O, E, I, X, R> Par<I, X, R>
where
    I: ConcurrentIter,
    X: Xap<I = I::Item, O = Result<O, E>>,
    X::Size: IntoSizePair,
    R: ParRunner,
{
    pub fn fallible_result(
        self,
    ) -> ParRes<I, O, E, X, Id<O>, <X::Size as IntoSizePair>::ThenOne, R> {
        let (iter, xap, exe, params) = self.destruct();
        let xap = XapRes::new(xap, Id::new());
        ParRes::new(iter, xap, exe, params)
    }
}
