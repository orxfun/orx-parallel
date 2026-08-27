use crate::infallible::recursive::utils;
use crate::infallible::recursive::xap_sync::XapSync;
use crate::{Par, ParDrain, ParThreadPool, ParUse, Params, infallible::Xap, runner::ParRunner};
use alloc::vec::Vec;
use core::cmp::Ordering;

struct Elem<T> {
    value: T,
    depth: usize,
    width: usize,
}

impl<T> Elem<T> {
    fn new(value: T, depth: usize, width: usize) -> Self {
        Self {
            value,
            depth,
            width,
        }
    }

    fn normalize_depths(elements: &mut [Self]) {
        if let Some(max_width) = elements.iter().map(|x| x.width).max() {
            let depth_coef = max_width + 1;
            for elem in elements {
                elem.depth = elem.depth * depth_coef + elem.width;
            }
        }
    }
}

impl<T> PartialEq for Elem<T> {
    fn eq(&self, other: &Self) -> bool {
        self.depth == other.depth && self.width == other.width
    }
}

impl<T> Eq for Elem<T> {}

impl<T> PartialOrd for Elem<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Elem<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.depth.cmp(&other.depth) {
            Ordering::Equal => {}
            ord => return ord,
        }
        debug_assert_ne!(self.width, other.width);
        self.width.cmp(&other.width)
    }
}

struct Local<I, O> {
    input: Vec<Elem<I>>,
    output: Vec<Elem<O>>,
}

impl<I, O> Local<I, O> {
    fn new() -> Self {
        Self {
            input: Default::default(),
            output: Default::default(),
        }
    }
}

pub fn collect<R, C, X, I, E>(
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

    let mut inputs: Vec<_> = iter
        .into_iter()
        .enumerate()
        .map(|(width, value)| Elem::new(value, 0, width))
        .collect();
    let mut result = Vec::new();

    let mut depth = 0;

    let par = inputs.par_drain(..).runner(&mut runner);
    let par = params.apply(par).use_slice(&mut data);
    par.for_each(|u, input| {
        let new_inputs = extend(&input.value)
            .into_iter()
            .enumerate()
            .map(|(width, value)| Elem::new(value, input.depth, width));
        u.input.extend(new_inputs);

        let values = xap.xap(input.value).into_iter();
        let outputs = values.map(|value| Elem::new(value, depth, input.depth));
        u.output.extend(outputs);
    });
    utils::into_outer_par(&mut inputs, &mut data, |x| &mut x.input, &mut runner);
    Elem::normalize_depths(&mut inputs);
    utils::into_outer_par(&mut result, &mut data, |x| &mut x.output, &mut runner);

    while !inputs.is_empty() {
        depth += 1;

        let par = inputs.par_drain(..).runner(&mut runner);
        let par = params.apply(par).use_slice(&mut data);
        par.for_each(|u, input| {
            let new_inputs = extend(&input.value)
                .into_iter()
                .enumerate()
                .map(|(width, value)| Elem::new(value, input.depth, width));
            u.input.extend(new_inputs);

            let values = xap.xap(input.value).into_iter();
            let outputs = values.map(|value| Elem::new(value, depth, input.depth));
            u.output.extend(outputs);
        });
        utils::into_outer_par(&mut inputs, &mut data, |x| &mut x.input, &mut runner);
        Elem::normalize_depths(&mut inputs);
        utils::into_outer_par(&mut result, &mut data, |x| &mut x.output, &mut runner);
    }

    result.sort();
    result.into_iter().map(|x| x.value).collect()
}
