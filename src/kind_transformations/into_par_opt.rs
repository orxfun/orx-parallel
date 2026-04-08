use crate::infallible::Par;
use crate::infallible::Xap;
use crate::infallible::xap_variants::Id;
use crate::option::ParOpt;
use crate::option::size_pairs::SizePairOpt;
use crate::runner::ParRunner;
use crate::sizes::IntoSizePair;
use orx_concurrent_iter::ConcurrentIter;

impl<O, I, X, R> Par<I, X, R>
where
    I: ConcurrentIter,
    X: Xap<I = I::Item, O = Option<O>>,
    X::Size: IntoSizePair,
    <X::Size as IntoSizePair>::ThenOne: SizePairOpt,
    R: ParRunner,
{
    pub fn fallible_option(self) -> ParOpt<I, O, X, Id<O>, <X::Size as IntoSizePair>::ThenOne, R> {
        let (iter, xap, exe, params) = self.destruct();
        ParOpt::new(iter, xap, Id::new(), exe, params)
    }
}
