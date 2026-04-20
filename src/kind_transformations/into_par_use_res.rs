use crate::infallible::Xap;
use crate::infallible::xap_variants::Id;
use crate::infallible_use::ParUse;
use crate::infallible_use::Use;
use crate::infallible_use::UseClone;
use crate::infallible_use::UseFun;
use crate::infallible_use::XapUse;
use crate::infallible_use::xap_variants::IdUse;
use crate::result::{ParRes, ParResIterCore};
use crate::result_use::ParUseRes;
use crate::runner::ParRunner;
use crate::sizes::{Size, SizePair};
use orx_concurrent_iter::ConcurrentIter;

// ParUse -> ParUseRes

impl<U, O, E, I, X, R> ParUse<U, I, X, R>
where
    U: Use,
    I: ConcurrentIter,
    X: XapUse<U = U::Item, I = I::Item, O = Result<O, E>>,
    R: ParRunner,
{
    pub fn into_fallible(
        self,
    ) -> ParUseRes<U, I, O, E, X, IdUse<Id<O>, U::Item>, <X::Size as Size>::IntoPair, R> {
        let (u, iter, xap, exe, params) = self.destruct();
        ParUseRes::new(u, iter, xap, IdUse::new(Id::new()), exe, params)
    }
}

// ParRes -> ParUseRes

impl<I, M, E, X1, X2, S, R> ParRes<I, M, E, X1, X2, S, R>
where
    I: ConcurrentIter,
    X1: Xap<I = I::Item, O = Result<M, E>>,
    X2: Xap<I = M>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
    R: ParRunner,
{
    pub fn using<U, F>(
        self,
        f: F,
    ) -> ParUseRes<UseFun<U, F>, I, M, E, IdUse<X1, U>, IdUse<X2, U>, S, R>
    where
        F: Fn(usize) -> U + Sync,
    {
        let (iter, x1, x2, exe, _, params) = self.destruct();
        let x1 = IdUse::<_, U>::new(x1);
        let x2 = IdUse::<_, U>::new(x2);
        let u = UseFun::new(f);
        ParUseRes::new(u, iter, x1, x2, exe, params)
    }

    pub fn using_clone<U>(
        self,
        u: U,
    ) -> ParUseRes<UseClone<U>, I, M, E, IdUse<X1, U>, IdUse<X2, U>, S, R>
    where
        U: Clone + Send,
    {
        let (iter, x1, x2, exe, _, params) = self.destruct();
        let x1 = IdUse::<_, U>::new(x1);
        let x2 = IdUse::<_, U>::new(x2);
        let u = UseClone::new(u);
        ParUseRes::new(u, iter, x1, x2, exe, params)
    }
}
