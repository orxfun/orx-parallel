use crate::infallible::Xap;
use crate::infallible::xap_variants::Id;
use crate::infallible_use::ParUse;
use crate::infallible_use::SizeInfUse;
use crate::infallible_use::Use;
use crate::infallible_use::UseClone;
use crate::infallible_use::UseFun;
use crate::infallible_use::XapUse;
use crate::infallible_use::xap_variants::IdUse;
use crate::option::{ParOpt, ParOptIterCore};
use crate::option_use::{ParUseOpt, SizePairUseOpt};
use crate::runner::ParRunner;
use crate::sizes::Size;
use crate::sizes::SizePair;
use orx_concurrent_iter::ConcurrentIter;

// ParUse -> ParUseOpt

impl<U, O, I, X, R> ParUse<U, I, X, R>
where
    U: Use,
    I: ConcurrentIter,
    X: XapUse<U = U::Item, I = I::Item, O = Option<O>>,
    <X::Size as Size>::IntoPair: SizePairUseOpt,
    R: ParRunner,
{
    pub fn into_optional(
        self,
    ) -> ParUseOpt<U, I, O, X, IdUse<Id<O>, U::Item>, <X::Size as Size>::IntoPair, R> {
        let (u, iter, xap, exe, params) = self.destruct();
        ParUseOpt::new(u, iter, xap, IdUse::new(Id::new()), exe, params)
    }
}

// ParOpt -> ParUseOpt

impl<I, M, X1, X2, S, R> ParOpt<I, M, X1, X2, S, R>
where
    I: ConcurrentIter,
    X1: Xap<I = I::Item, O = Option<M>>,
    X2: Xap<I = M>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size> + SizePairUseOpt,
    R: ParRunner,
    X1::Size: SizeInfUse,
    X2::Size: SizeInfUse,
{
    pub fn using<U, F>(
        self,
        f: F,
    ) -> ParUseOpt<UseFun<U, F>, I, M, IdUse<X1, U>, IdUse<X2, U>, S, R>
    where
        F: Fn(usize) -> U + Sync,
    {
        let (iter, x1, x2, exe, _, params) = self.destruct();
        let x1 = IdUse::<_, U>::new(x1);
        let x2 = IdUse::<_, U>::new(x2);
        let u = UseFun::new(f);
        ParUseOpt::new(u, iter, x1, x2, exe, params)
    }

    pub fn using_clone<U>(
        self,
        u: U,
    ) -> ParUseOpt<UseClone<U>, I, M, IdUse<X1, U>, IdUse<X2, U>, S, R>
    where
        U: Clone + Send,
    {
        let (iter, x1, x2, exe, _, params) = self.destruct();
        let x1 = IdUse::<_, U>::new(x1);
        let x2 = IdUse::<_, U>::new(x2);
        let u = UseClone::new(u);
        ParUseOpt::new(u, iter, x1, x2, exe, params)
    }
}
