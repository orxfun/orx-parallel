use crate::option::recursive::utils;
use crate::option::recursive::xap_sync::XapSync;
use crate::{Par, ParDrain, ParThreadPool, ParUse, Params, infallible::Xap, runner::ParRunner};
use alloc::vec::Vec;

struct Local<I, B> {
    input: Vec<I>,
    fold: B,
    failed: bool,
}

impl<I, B> Local<I, B> {
    fn new(init: B) -> Self {
        Self {
            input: Default::default(),
            fold: init,
            failed: false,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn fold<R, C, M, X1, X2, B, Id, F, I, E>(
    mut runner: R,
    params: Params,
    iter: C,
    x1: X1,
    x2: X2,
    extend: E,
    init: Id,
    f: F,
) -> Option<Vec<B>>
where
    R: ParRunner,
    C: IntoIterator,
    C::Item: Clone + Send + Sync,
    X1: Xap<I = C::Item, O = Option<M>>,
    X2: Xap<I = M>,
    I: IntoIterator<Item = C::Item>,
    E: Fn(&C::Item) -> I + Send + Sync,
    B: Send + Sync,
    Id: Fn() -> B + Sync,
    F: Fn(&mut B, X2::O) + Copy + Send + Sync,
    M: Send + Sync,
{
    let x1 = XapSync::new(x1);
    let x2 = XapSync::new(x2);
    let max_threads: usize = runner.pool().max_num_threads().into();

    let mut data: Vec<_> = (0..max_threads).map(|_| Local::new(init())).collect();
    let mut outer: Vec<_> = iter.into_iter().collect();

    loop {
        let par = outer.par_drain(..).runner(&mut runner);
        let par = params.apply(par).use_slice(&mut data);

        par.for_each(|u, i| {
            if u.failed {
                return;
            }
            let i_clone = i.clone();
            match utils::split(&x1, i) {
                Some(ms) => {
                    u.input.extend(extend(&i_clone));
                    for m in ms {
                        for out in x2.xap(m) {
                            f(&mut u.fold, out);
                        }
                    }
                }
                None => u.failed = true,
            }
        });

        if data.iter().any(|d| d.failed) {
            return None;
        }

        let len = data.iter().map(|x| x.input.len()).sum();
        utils::into_outer(&mut outer, len, data.iter_mut().map(|x| &mut x.input));

        if outer.is_empty() {
            break;
        }
    }

    Some(data.into_iter().map(|x| x.fold).collect())
}
