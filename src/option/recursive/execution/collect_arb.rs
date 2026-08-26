use crate::option::recursive::utils;
use crate::option::recursive::xap_sync::XapSync;
use crate::{Par, ParDrain, ParThreadPool, ParUse, Params, infallible::Xap, runner::ParRunner};
use alloc::vec::Vec;

struct Local<I, O> {
    input: Vec<I>,
    output: Vec<O>,
    failed: bool,
}

impl<I, O> Local<I, O> {
    fn new() -> Self {
        Self {
            input: Default::default(),
            output: Default::default(),
            failed: false,
        }
    }
}

pub fn collect_arb<R, C, M, X1, X2, I, E>(
    mut runner: R,
    params: Params,
    iter: C,
    x1: X1,
    x2: X2,
    extend: E,
) -> Option<Vec<Vec<X2::O>>>
where
    R: ParRunner,
    C: IntoIterator,
    C::Item: Clone + Send + Sync,
    X1: Xap<I = C::Item, O = Option<M>>,
    X2: Xap<I = M>,
    I: IntoIterator<Item = C::Item>,
    E: Fn(&C::Item) -> I + Send + Sync,
    X2::O: Send + Sync,
    M: Send + Sync,
{
    let x1 = XapSync::new(x1);
    let x2 = XapSync::new(x2);
    let max_threads: usize = runner.pool().max_num_threads().into();

    let mut data: Vec<_> = (0..max_threads).map(|_| Local::new()).collect();
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
                        u.output.extend(x2.xap(m));
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

    Some(data.into_iter().map(|x| x.output).collect())
}
