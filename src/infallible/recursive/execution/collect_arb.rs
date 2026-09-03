use crate::ParExtend;
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

pub fn collect_arb<R, C, X, I, E, P>(
    mut runner: R,
    params: Params,
    iter: C,
    xap: X,
    extend: E,
    dst: &mut P,
) where
    R: ParRunner,
    C: IntoIterator,
    X: Xap<I = C::Item>,
    I: IntoIterator<Item = X::I>,
    E: Fn(&X::I) -> I + Send + Copy,
    X::O: Send,
    X::I: Send,
    P: ParExtend<X::O>,
{
    let max_threads: usize = runner.pool().max_num_threads().into();

    let mut data: Vec<_> = (0..max_threads).map(|_| Local::new()).collect();

    let mut inputs: Vec<_> = iter.into_iter().collect();

    let par = inputs.par_drain(..).runner(&mut runner);
    let par = params.apply(par).use_slice(&mut data);

    par.for_each(move |u, i| {
        u.input.extend(extend(&i));
        u.output.extend(xap.xap(i));
    });
    utils::into_outer_par(&mut inputs, &mut data, |x| &mut x.input, &mut runner);
    dst.extend(data.iter_mut().flat_map(|x| x.output.drain(..)));

    while !inputs.is_empty() {
        let par = inputs.par_drain(..).runner(&mut runner);
        let par = params.apply(par).use_slice(&mut data);

        par.for_each(move |u, i| {
            u.input.extend(extend(&i));
            u.output.extend(xap.xap(i));
        });

        utils::into_outer_par(&mut inputs, &mut data, |x| &mut x.input, &mut runner);
        dst.extend(data.iter_mut().flat_map(|x| x.output.drain(..)));
    }
}
