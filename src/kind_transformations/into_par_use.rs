use crate::infallible::{Par, ParIterDestruct, Xap};
use crate::infallible_use::ParUse;
use crate::infallible_use::SizeInfUse;
use crate::infallible_use::UseClone;
use crate::infallible_use::UseFun;
use crate::infallible_use::xap_variants::IdUse;
use crate::runner::ParRunner;
use orx_concurrent_iter::ConcurrentIter;

impl<I, X, R> Par<I, X, R>
where
    I: ConcurrentIter,
    X: Xap<I = I::Item>,
    R: ParRunner,
    X::Size: SizeInfUse,
{
    pub fn using<U, F>(self, f: F) -> ParUse<UseFun<U, F>, I, IdUse<X, U>, R>
    where
        F: Fn(usize) -> U + Sync,
    {
        let (iter, xap, exe, params) = self.destruct();
        let using = UseFun::new(f);
        let xap: IdUse<X, U> = IdUse::new(xap);
        ParUse::new(using, iter, xap, exe, params)
    }

    pub fn using_clone<U>(self, u: U) -> ParUse<UseClone<U>, I, IdUse<X, U>, R>
    where
        U: Clone + Send,
    {
        let (iter, xap, exe, params) = self.destruct();
        let using = UseClone::new(u);
        let xap: IdUse<X, U> = IdUse::new(xap);
        ParUse::new(using, iter, xap, exe, params)
    }
}
