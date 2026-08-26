use crate::infallible::recursive::utils;
use crate::{Par, ParDrain, ParThreadPool, ParUse, Params, infallible::Xap, runner::ParRunner};
use alloc::vec::Vec;

struct Local<I, O> {
    input: Vec<I>,
    output: Vec<O>,
}

impl<I, O> Local<I, O> {
    fn new() -> Self {
        Self {
            input: Default::default(),
            output: Default::default(),
        }
    }
}

pub fn collect_arb<R, C, X, I, E>(
    runner: R,
    params: Params,
    iter: C,
    x: X,
    extend: E,
) -> Vec<Vec<X::O>>
where
    R: ParRunner + Clone,
    C: IntoIterator,
    X: Xap<I = C::Item>,
    I: IntoIterator<Item = X::I>,
    E: Fn(&X::I) -> I + Send + Sync,
    // TODO: revisit these requirements
    X::O: Send + Sync,
    X::I: Send + Sync,
    X: Send + Sync,
{
    let max_threads: usize = runner.pool().max_num_threads().into();

    let mut data: Vec<_> = (0..max_threads).map(|_| Local::new()).collect();

    let mut outer: Vec<_> = iter.into_iter().collect();

    let par = outer.par_drain(..).runner(runner.clone());
    let par = params.apply(par).use_slice(&mut data);

    par.for_each(|u, i| {
        u.input.extend(extend(&i));
        u.output.extend(x.xap(i));
    });
    let len = data.iter().map(|x| x.input.len()).sum();
    utils::into_outer(&mut outer, len, data.iter_mut().map(|x| &mut x.input));

    while !outer.is_empty() {
        let par = outer.par_drain(..).runner(runner.clone());
        let par = params.apply(par).use_slice(&mut data);

        par.for_each(|u, i| {
            u.input.extend(extend(&i));
            u.output.extend(x.xap(i));
        });

        let len = data.iter().map(|x| x.input.len()).sum();
        utils::into_outer(&mut outer, len, data.iter_mut().map(|x| &mut x.input));
    }

    data.into_iter().map(|x| x.output).collect()
}
