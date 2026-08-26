use crate::infallible::recursive::utils;
use crate::infallible::recursive::xap_sync::XapSync;
use crate::{Par, ParDrain, ParThreadPool, ParUse, Params, infallible::Xap, runner::ParRunner};
use alloc::vec::Vec;

struct Local<I, B> {
    input: Vec<I>,
    fold: B,
}

impl<I, B> Local<I, B> {
    fn new(init: B) -> Self {
        Self {
            input: Default::default(),
            fold: init,
        }
    }
}

pub fn fold<R, C, X, B, Id, F, I, E>(
    mut runner: R,
    params: Params,
    iter: C,
    xap: X,
    extend: E,
    init: Id,
    f: F,
) -> Vec<B>
where
    R: ParRunner,
    C: IntoIterator,
    X: Xap<I = C::Item>,
    I: IntoIterator<Item = X::I>,
    E: Fn(&X::I) -> I + Send + Sync,
    B: Send + Sync,
    Id: Fn() -> B + Sync,
    F: Fn(&mut B, X::O) + Copy + Send + Sync,
    X::I: Send + Sync,
{
    let xap = XapSync::new(xap);
    let max_threads: usize = runner.pool().max_num_threads().into();

    let mut data: Vec<_> = (0..max_threads)
        .map(|_| Local::<X::I, B>::new(init()))
        .collect();

    let mut outer: Vec<_> = iter.into_iter().collect();

    let par = outer.par_drain(..).runner(&mut runner);
    let par = params.apply(par).use_slice(&mut data);

    par.for_each(|u, i| {
        u.input.extend(extend(&i));

        for i in xap.xap(i) {
            f(&mut u.fold, i);
        }
    });
    let len = data.iter().map(|x| x.input.len()).sum();
    utils::into_outer(&mut outer, len, data.iter_mut().map(|x| &mut x.input));

    while !outer.is_empty() {
        let par = outer.par_drain(..).runner(&mut runner);
        let par = params.apply(par).use_slice(&mut data);

        par.for_each(|u, i| {
            u.input.extend(extend(&i));

            for i in xap.xap(i) {
                f(&mut u.fold, i);
            }
        });

        let len = data.iter().map(|x| x.input.len()).sum();
        utils::into_outer(&mut outer, len, data.iter_mut().map(|x| &mut x.input));
    }

    data.into_iter().map(|x| x.fold).collect()
}
