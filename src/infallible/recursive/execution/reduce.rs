use crate::infallible::recursive::utils;
use crate::infallible::recursive::xap_sync::XapSync;
use crate::{Par, ParDrain, ParThreadPool, ParUse, Params, infallible::Xap, runner::ParRunner};
use alloc::vec::Vec;

pub fn reduce<R, C, X, F, I, E>(
    mut runner: R,
    params: Params,
    iter: C,
    xap: X,
    extend: E,
    f: F,
) -> Option<X::O>
where
    R: ParRunner,
    C: IntoIterator,
    X: Xap<I = C::Item>,
    I: IntoIterator<Item = X::I>,
    E: Fn(&X::I) -> I + Send + Sync,
    F: Fn(X::O, X::O) -> X::O + Send + Copy,
    X::O: Send,
    X::I: Send + Sync,
{
    let xap = XapSync::new(xap);
    let max_threads: usize = runner.pool().max_num_threads().into();

    let mut data: Vec<_> = (0..max_threads).map(|_| Vec::<X::I>::new()).collect();

    let mut outer: Vec<_> = iter.into_iter().collect();

    let par = outer.par_drain(..).runner(&mut runner);
    let par = params.apply(par).use_slice(&mut data);

    let mut result = par
        .flat_map(|u, i| {
            u.extend(extend(&i));
            xap.xap(i)
        })
        .reduce(move |_, a, b| f(a, b));
    let len = data.iter().map(|x| x.len()).sum();
    utils::inputs_into_outer(&mut outer, len, &mut data);

    while !outer.is_empty() {
        let par = outer.par_drain(..).runner(&mut runner);
        let par = params.apply(par).use_slice(&mut data);

        let result_wave = par
            .flat_map(|u, i| {
                u.extend(extend(&i));
                xap.xap(i)
            })
            .reduce(move |_, a, b| f(a, b));

        result = match (result, result_wave) {
            (Some(a), Some(b)) => Some(f(a, b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };

        let len = data.iter().map(|x| x.len()).sum();
        utils::inputs_into_outer(&mut outer, len, &mut data);
    }

    result
}
