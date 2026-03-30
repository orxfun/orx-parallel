use crate::infallible::Par;
use crate::infallible::Xap;
use crate::into_fallible::IntoXapRes;
use crate::result::ParRes;
use crate::runner::ParRunner;
use orx_concurrent_iter::ConcurrentIter;

impl<O, E, I, X, R> Par<I, X, R>
where
    I: ConcurrentIter,
    X: Xap<I = I::Item, O = Result<O, E>> + IntoXapRes,
    R: ParRunner,
{
    pub fn into_result(self) -> ParRes<I, X::XapRes, R> {
        let (iter, xap, exe, params) = self.destruct();
        let xap = xap.into_xap_res();
        ParRes::new(iter, xap, exe, params)
    }
}
