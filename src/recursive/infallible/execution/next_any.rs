use crate::recursive::infallible::utils;
use crate::{Par, ParDrain, ParThreadPool, ParUse, Params, infallible::Xap, runner::ParRunner};
use alloc::vec::Vec;

pub fn next_any<R, C, X, I, E>(runner: R, params: Params, iter: C, x: X, extend: E) -> Option<X::O>
where
    R: ParRunner + Clone,
    C: IntoIterator,
    X: Xap<I = C::Item>,
    I: IntoIterator<Item = X::I>,
    E: Fn(&X::I) -> I + Send + Sync,
    // TODO: revisit these requirements
    X::O: Send,
    X::I: Send + Sync,
    X: Send + Sync,
{
    let max_threads: usize = runner.pool().max_num_threads().into();

    let mut data: Vec<_> = (0..max_threads).map(|_| Vec::<X::I>::new()).collect();

    let mut outer: Vec<_> = iter.into_iter().collect();

    let par = outer.par_drain(..).runner(runner.clone());
    let par = params.apply(par).use_slice(&mut data);

    let result = par
        .flat_map(|u, i| {
            u.extend(extend(&i));
            x.xap(i)
        })
        .first();

    match result.is_some() {
        true => result,
        false => {
            let len = data.iter().map(|x| x.len()).sum();
            utils::into_outer(&mut outer, len, &mut data);

            while !outer.is_empty() {
                let par = outer.par_drain(..).runner(runner.clone());
                let par = params.apply(par).use_slice(&mut data);

                let result = par
                    .flat_map(|u, i| {
                        u.extend(extend(&i));
                        x.xap(i)
                    })
                    .first();

                if result.is_some() {
                    return result;
                }

                let len = data.iter().map(|x| x.len()).sum();
                utils::into_outer(&mut outer, len, &mut data);
            }

            None
        }
    }
}
