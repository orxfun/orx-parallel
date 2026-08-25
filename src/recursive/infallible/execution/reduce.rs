use crate::recursive::infallible::utils;
use crate::{Par, ParDrain, ParThreadPool, ParUse, Params, infallible::Xap, runner::ParRunner};
use alloc::vec::Vec;
use orx_concurrent_iter::ConcurrentIter;

pub fn reduce<R, C, X, F, I, E>(
    runner: R,
    params: Params,
    iter: C,
    x: X,
    extend: E,
    f: F,
) -> Option<X::O>
where
    R: ParRunner + Clone,
    C: ConcurrentIter,
    X: Xap<I = C::Item>,
    I: IntoIterator<Item = X::I>,
    E: Fn(&X::I) -> I + Send + Sync,
    F: Fn(X::O, X::O) -> X::O + Sync,
    // TODO: revisit these requirements
    X::O: Send,
    X::I: Send + Sync,
    X: Send + Sync,
{
    let max_threads: usize = runner.pool().max_num_threads().into();

    let mut data: Vec<_> = (0..max_threads).map(|_| Vec::<X::I>::new()).collect();

    let mut outer: Vec<_> = iter.into_seq_iter().collect();

    let par = outer.par_drain(..).runner(runner.clone());
    let par = params.configure_par(par).use_slice(&mut data);

    let mut result = par
        .flat_map(|u, i| {
            u.extend(extend(&i));
            x.xap(i)
        })
        .reduce(|_, a, b| f(a, b));
    utils::into_outer(&mut outer, &mut data);

    while !outer.is_empty() {
        let par = outer.par_drain(..).runner(runner.clone());
        let par = params.configure_par(par).use_slice(&mut data);

        let result_wave = par
            .flat_map(|u, i| {
                u.extend(extend(&i));
                x.xap(i)
            })
            .reduce(|_, a, b| f(a, b));

        result = match (result, result_wave) {
            (Some(a), Some(b)) => Some(f(a, b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    }

    result
}
