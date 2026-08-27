use crate::infallible::recursive::utils;
use crate::infallible::recursive::xap_sync::XapSync;
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
    mut runner: R,
    params: Params,
    iter: C,
    xap: X,
    extend: E,
) -> Vec<X::O>
where
    R: ParRunner,
    C: IntoIterator,
    X: Xap<I = C::Item>,
    I: IntoIterator<Item = X::I>,
    E: Fn(&X::I) -> I + Send + Sync,
    X::O: Send + Sync,
    X::I: Send + Sync,
{
    let xap = XapSync::new(xap);
    let max_threads: usize = runner.pool().max_num_threads().into();

    let mut data: Vec<_> = (0..max_threads).map(|_| Local::new()).collect();

    let mut outer: Vec<_> = iter.into_iter().collect();
    let mut result = Vec::new();

    let par = outer.par_drain(..).runner(&mut runner);
    let par = params.apply(par).use_slice(&mut data);

    par.for_each(|u, i| {
        u.input.extend(extend(&i));
        u.output.extend(xap.xap(i));
    });
    let len = data.iter().map(|x| x.input.len()).sum();
    utils::inputs_into_outer(&mut outer, len, data.iter_mut().map(|x| &mut x.input));

    let len = data.iter().map(|x| x.output.len()).sum();
    utils::outputs_into_outer(&mut result, len, data.iter_mut().map(|x| &mut x.output));

    while !outer.is_empty() {
        let par = outer.par_drain(..).runner(&mut runner);
        let par = params.apply(par).use_slice(&mut data);

        par.for_each(|u, i| {
            u.input.extend(extend(&i));
            u.output.extend(xap.xap(i));
        });

        let len = data.iter().map(|x| x.input.len()).sum();
        utils::inputs_into_outer(&mut outer, len, data.iter_mut().map(|x| &mut x.input));

        let len = data.iter().map(|x| x.output.len()).sum();
        utils::outputs_into_outer(&mut result, len, data.iter_mut().map(|x| &mut x.output));
    }

    result
}
