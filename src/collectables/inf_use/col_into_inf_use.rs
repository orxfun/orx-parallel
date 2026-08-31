use crate::collectables::Collectable;
use crate::collectables::alg::merge_collected::merge_arb;
use crate::collectables::inf::ColIntoInf;
use crate::infallible_use::{ParRunnerInfallibleUse, ParUseCore, ParUseIter, XapUse};
use crate::use_var::Use;
use orx_concurrent_iter::ConcurrentIter;

pub trait ColIntoInfUse<T>: Sized {
    fn inf_use_col_into<U, I, X, R>(dst: &mut Self, par: ParUseIter<U, I, X, R>)
    where
        U: Use,
        I: ConcurrentIter,
        X: XapUse<U = U::Item, I = I::Item, O = T>,
        R: ParRunnerInfallibleUse,
        T: Send;

    // newcol

    fn inf_use_arb_col_into<U, I, X, R>(dst: &mut Self, par: ParUseIter<U, I, X, R>)
    where
        U: Use,
        I: ConcurrentIter,
        X: XapUse<U = U::Item, I = I::Item, O = T>,
        R: ParRunnerInfallibleUse,
        T: Send,
        Self: ColIntoInf<T> + Collectable<T>,
    {
        let (u, iter, x, mut exe, params) = par.destruct();
        let results =
            exe.collect_arb::<_, _, _, <Self as ColIntoInf<T>>::ThreadColArb>(params, u, iter, x);
        merge_arb(results, dst);
    }
}
