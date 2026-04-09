use crate::infallible::Par;
use crate::infallible::Xap;
use crate::infallible_use::ParUse;
use crate::infallible_use::SizeInfUse;
use crate::infallible_use::UseClone;
use crate::infallible_use::UseFun;
use crate::infallible_use::xap_variants::Id;
use crate::runner::ParRunner;
use orx_concurrent_iter::ConcurrentIter;

impl<O, E, I, X, R> Par<I, X, R>
where
    I: ConcurrentIter,
    X: Xap<I = I::Item, O = Result<O, E>>,
    R: ParRunner,
    X::Size: SizeInfUse,
{
    pub fn using<U, F>(self, f: F) -> ParUse<UseFun<U, F>, I, Id<X, U>, R>
    where
        F: Fn(usize) -> U + Sync,
    {
        let (iter, xap, exe, params) = self.destruct();
        let using = UseFun::new(f);
        let xap: Id<X, U> = Id::new(xap);
        ParUse::new(using, iter, xap, exe, params)
    }

    pub fn using_clone<U>(self, u: U) -> ParUse<UseClone<U>, I, Id<X, U>, R>
    where
        U: Clone + Send,
    {
        let (iter, xap, exe, params) = self.destruct();
        let using = UseClone::new(u);
        let xap: Id<X, U> = Id::new(xap);
        ParUse::new(using, iter, xap, exe, params)
    }
}
