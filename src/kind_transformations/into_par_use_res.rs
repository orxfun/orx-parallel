use crate::infallible::xap_variants::Id;
use crate::infallible_use::ParUse;
use crate::infallible_use::Use;
use crate::infallible_use::XapUse;
use crate::infallible_use::xap_variants::IdUse;
use crate::result_use::ParUseRes;
use crate::result_use::SizePairUseRes;
use crate::runner::ParRunner;
use crate::sizes::IntoSizePair;
use orx_concurrent_iter::ConcurrentIter;

// ParUse -> ParUseRes

impl<U, O, E, I, X, R> ParUse<U, I, X, R>
where
    U: Use,
    I: ConcurrentIter,
    X: XapUse<U = U::Item, I = I::Item, O = Result<O, E>>,
    X::Size: IntoSizePair,
    <X::Size as IntoSizePair>::ThenOne: SizePairUseRes,
    R: ParRunner,
{
    pub fn fallible_result(
        self,
    ) -> ParUseRes<U, I, O, E, X, IdUse<Id<O>, U::Item>, <X::Size as IntoSizePair>::ThenOne, R>
    {
        let (u, iter, xap, exe, params) = self.destruct();
        ParUseRes::new(u, iter, xap, IdUse::new(Id::new()), exe, params)
    }
}
