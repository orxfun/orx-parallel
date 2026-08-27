use crate::EnumerateParUse;
use crate::infallible::recursive::utils;
use crate::infallible::recursive::xap_sync::XapSync;
use crate::{Par, ParDrain, ParThreadPool, ParUse, Params, infallible::Xap, runner::ParRunner};
use alloc::vec::Vec;
use core::cmp::Ordering;

struct Item<O> {
    value: O,
    depth: usize,
    width: usize,
}

impl<O> Item<O> {
    fn new(value: O, depth: usize, width: usize) -> Self {
        Self {
            value,
            depth,
            width,
        }
    }
}

impl<O> PartialEq for Item<O> {
    fn eq(&self, other: &Self) -> bool {
        self.depth == other.depth && self.width == other.width
    }
}

impl<O> Eq for Item<O> {}

impl<O> PartialOrd for Item<O> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<O> Ord for Item<O> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.depth.cmp(&other.depth) {
            Ordering::Equal => {}
            ord => return ord,
        }
        self.width.cmp(&other.width)
    }
}

struct Local<I, O> {
    input: Vec<I>,
    output: Vec<Item<O>>,
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

    let mut outer: Vec<_> = iter.into_iter().collect();
    let mut result = Vec::new();

    let mut depth = 0;

    let par = outer.par_drain(..).runner(&mut runner);
    let par = params.apply(par).use_slice(&mut data).enumerate();
    par.for_each(|u, (width, i)| {
        u.input.extend(extend(&i));
        let values = xap.xap(i).into_iter();
        let items = values.map(|value| Item::new(value, depth, width));
        u.output.extend(items);
    });
    utils::into_outer_par(&mut outer, &mut data, |x| &mut x.input, &mut runner);
    utils::into_outer_par(&mut result, &mut data, |x| &mut x.output, &mut runner);

    while !outer.is_empty() {
        depth += 1;

        let par = outer.par_drain(..).runner(&mut runner);
        let par = params.apply(par).use_slice(&mut data).enumerate();
        par.for_each(|u, (width, i)| {
            u.input.extend(extend(&i));
            let values = xap.xap(i).into_iter();
            let items = values.map(|value| Item::new(value, depth, width));
            u.output.extend(items);
        });
        utils::into_outer_par(&mut outer, &mut data, |x| &mut x.input, &mut runner);
        utils::into_outer_par(&mut result, &mut data, |x| &mut x.output, &mut runner);
    }

    result.sort();
    result.into_iter().map(|x| x.value).collect()
}
