use crate::collectables::Collectable;
use crate::collectables::alg::merge_collected::merge_arb;
use crate::collectables::inf::ColIntoInf;
use crate::infallible::Xap;
use crate::option::{ParOptionCore, ParOptionIter, ParRunnerOpt};
use crate::sizes::SizePair;
use orx_concurrent_iter::ConcurrentIter;

pub trait ColIntoOpt<T>: Sized {
    fn opt_col_into<I, M, X1, X2, S, R>(
        dst: &mut Self,
        par: ParOptionIter<I, M, X1, X2, S, R>,
    ) -> Option<()>
    where
        I: ConcurrentIter,
        X1: Xap<I = I::Item, O = Option<M>>,
        X2: Xap<I = M, O = T>,
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerOpt,
        T: Send;

    fn opt_arb_col_into<I, M, X1, X2, S, R>(
        dst: &mut Self,
        par: ParOptionIter<I, M, X1, X2, S, R>,
    ) -> Option<()>
    where
        I: ConcurrentIter,
        X1: Xap<I = I::Item, O = Option<M>>,
        X2: Xap<I = M, O = T>,
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerOpt,
        T: Send,
        Self: ColIntoInf<T> + Collectable<T>,
    {
        let (iter, x1, x2, mut exe, s, params) = par.destruct();
        let results = exe.collect_arb::<_, _, _, _, _, <Self as ColIntoInf<T>>::ThreadColArb>(
            s, params, iter, x1, x2,
        );
        results.map(|results| merge_arb(results, dst))
    }
}
