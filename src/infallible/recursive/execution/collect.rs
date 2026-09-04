use crate::ParExtend;
use crate::infallible::recursive::execution::elem::{ElemIn, ElemOut};
use crate::infallible::recursive::utils;
use crate::{Par, ParDrain, ThreadPool, ParUse, Params, infallible::Xap, runner::ParRunner};
use alloc::vec::Vec;

struct Local<I, O> {
    input: Vec<ElemIn<I>>,
    output: Vec<ElemOut<O>>,
}

impl<I, O> Local<I, O> {
    fn new() -> Self {
        Self {
            input: Default::default(),
            output: Default::default(),
        }
    }
}

pub fn collect<R, C, X, I, E, P>(
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

    let mut inputs: Vec<_> = iter
        .into_iter()
        .enumerate()
        .map(|(ch, value)| ElemIn::new(value, 0, ch))
        .collect();
    let mut result = Vec::new();

    let mut depth = 0;

    let par = inputs.par_drain(..).runner(&mut runner);
    let par = params.apply(par).use_slice(&mut data);
    par.for_each(move |u, input| {
        let new_inputs = extend(&input.value)
            .into_iter()
            .enumerate()
            .map(|(ch, value)| ElemIn::new(value, 0, ch));
        u.input.extend(new_inputs);

        let values = xap.xap(input.value).into_iter();
        let outputs = values.map(|value| ElemOut::new(value, 0, input.child_idx));
        u.output.extend(outputs);
    });
    utils::into_outer_par(&mut inputs, &mut data, |x| &mut x.input, &mut runner);
    ElemIn::normalize_parent_indices(&mut inputs);
    utils::into_outer_par(&mut result, &mut data, |x| &mut x.output, &mut runner);

    while !inputs.is_empty() {
        depth += 1;

        let par = inputs.par_drain(..).runner(&mut runner);
        let par = params.apply(par).use_slice(&mut data);
        par.for_each(move |u, input| {
            let new_inputs = extend(&input.value)
                .into_iter()
                .enumerate()
                .map(|(width, value)| ElemIn::new(value, input.parent_idx, width));
            u.input.extend(new_inputs);

            let values = xap.xap(input.value).into_iter();
            let outputs = values.map(|value| ElemOut::new(value, depth, input.parent_idx));
            u.output.extend(outputs);
        });
        utils::into_outer_par(&mut inputs, &mut data, |x| &mut x.input, &mut runner);
        ElemIn::normalize_parent_indices(&mut inputs);
        utils::into_outer_par(&mut result, &mut data, |x| &mut x.output, &mut runner);
    }

    result.sort();
    dst.extend(result.into_iter().map(|x| x.value));
}
