use crate::collectables::Collectable;
use crate::collectables::alg::merge_collected::merge_arb;
use crate::collectables::inf::ColIntoInf;
use crate::infallible_use::XapUse;
use crate::option_use::{ParRunnerUseOpt, ParUseOptionCore, ParUseOptionIter};
use crate::sizes::SizePair;
use crate::use_var::Use;
use orx_concurrent_iter::ConcurrentIter;

pub trait ColIntoOptUse<T>: Sized {
    fn opt_use_col_into<U, I, M, X1, X2, S, R>(
        dst: &mut Self,
        par: ParUseOptionIter<U, I, M, X1, X2, S, R>,
    ) -> Option<()>
    where
        U: Use,
        I: ConcurrentIter,
        X1: XapUse<U = U::Item, I = I::Item, O = Option<M>>,
        X2: XapUse<U = U::Item, I = M, O = T>,
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerUseOpt,
        T: Send;

    fn opt_use_arb_col_into<U, I, M, X1, X2, S, R>(
        dst: &mut Self,
        par: ParUseOptionIter<U, I, M, X1, X2, S, R>,
    ) -> Option<()>
    where
        U: Use,
        I: ConcurrentIter,
        X1: XapUse<U = U::Item, I = I::Item, O = Option<M>>,
        X2: XapUse<U = U::Item, I = M, O = T>,
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerUseOpt,
        T: Send,
        Self: ColIntoInf<T> + Collectable<T>,
    {
        let (u, iter, x1, x2, mut exe, s, params) = par.destruct();
        let results = exe.collect_arb::<_, _, _, _, _, _, <Self as ColIntoInf<T>>::ThreadColArb>(
            s, params, u, iter, x1, x2,
        );
        results.map(|results| merge_arb(results, dst))
    }
}
