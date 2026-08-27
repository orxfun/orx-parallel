use crate::infallible::recursive::execution::elem::Elem;
use crate::infallible::recursive::utils;
use crate::infallible::recursive::xap_sync::XapSync;
use crate::{Par, ParDrain, ParThreadPool, ParUse, Params, infallible::Xap, runner::ParRunner};
use alloc::vec::Vec;

pub fn next<R, C, X, I, E>(
    mut runner: R,
    params: Params,
    iter: C,
    xap: X,
    extend: E,
) -> Option<X::O>
where
    R: ParRunner,
    C: IntoIterator,
    X: Xap<I = C::Item>,
    I: IntoIterator<Item = X::I>,
    E: Fn(&X::I) -> I + Send + Sync,
    X::O: Send,
    X::I: Send + Sync,
{
    let xap = XapSync::new(xap);
    let max_threads: usize = runner.pool().max_num_threads().into();

    let mut data: Vec<_> = (0..max_threads).map(|_| Vec::new()).collect();

    let mut inputs: Vec<_> = iter
        .into_iter()
        .enumerate()
        .map(|(width, value)| Elem::new(value, 0, width))
        .collect();

    let par = inputs.par_drain(..).runner(&mut runner);
    let par = params.apply(par).use_slice(&mut data);

    let result = par
        .flat_map(|u, input| {
            let new_inputs = extend(&input.value)
                .into_iter()
                .enumerate()
                .map(|(width, value)| Elem::new(value, input.depth, width));
            u.extend(new_inputs);
            xap.xap(input.value)
        })
        .first();

    match result.is_some() {
        true => result,
        false => {
            utils::into_outer_par(&mut inputs, &mut data, |x| x, &mut runner);
            Elem::normalize_depths(&mut inputs);
            inputs.sort_unstable_by_key(|x| x.depth);

            while !inputs.is_empty() {
                let par = inputs.par_drain(..).runner(&mut runner);
                let par = params.apply(par).use_slice(&mut data);

                let result = par
                    .flat_map(|u, input| {
                        let new_inputs = extend(&input.value)
                            .into_iter()
                            .enumerate()
                            .map(|(width, value)| Elem::new(value, input.depth, width));
                        u.extend(new_inputs);
                        xap.xap(input.value)
                    })
                    .first();

                if result.is_some() {
                    return result;
                }
                utils::into_outer_par(&mut inputs, &mut data, |x| x, &mut runner);
                Elem::normalize_depths(&mut inputs);
                inputs.sort_unstable_by_key(|x| x.depth);
            }

            None
        }
    }
}
