use crate::option::recursive::utils;
use crate::option::recursive::xap_sync::XapSync;
use crate::{Par, ParDrain, ParThreadPool, ParUse, Params, infallible::Xap, runner::ParRunner};
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};

pub fn reduce<R, C, M, X1, X2, F, I, E>(
    mut runner: R,
    params: Params,
    iter: C,
    x1: X1,
    x2: X2,
    extend: E,
    f: F,
) -> Option<Option<X2::O>>
where
    R: ParRunner,
    C: IntoIterator,
    C::Item: Clone + Send + Sync,
    X1: Xap<I = C::Item, O = Option<M>>,
    X2: Xap<I = M>,
    I: IntoIterator<Item = C::Item>,
    E: Fn(&C::Item) -> I + Send + Sync,
    F: Fn(X2::O, X2::O) -> X2::O + Send + Copy,
    X2::O: Send,
    M: Send + Sync,
{
    let x1 = XapSync::new(x1);
    let x2 = XapSync::new(x2);
    let max_threads: usize = runner.pool().max_num_threads().into();

    let mut data: Vec<_> = (0..max_threads).map(|_| Vec::<C::Item>::new()).collect();
    let mut outer: Vec<_> = iter.into_iter().collect();
    let mut result: Option<X2::O> = None;
    let failed = AtomicBool::new(false);

    loop {
        let par = outer.par_drain(..).runner(&mut runner);
        let par = params.apply(par).use_slice(&mut data);

        let wave_result = par
            .flat_map(|u, i| {
                let i_clone = i.clone();
                match utils::split(&x1, i) {
                    Some(ms) => {
                        u.extend(extend(&i_clone));
                        ms.into_iter().flat_map(|m| x2.xap(m)).collect::<Vec<_>>()
                    }
                    None => {
                        failed.store(true, Ordering::Relaxed);
                        Vec::new()
                    }
                }
            })
            .reduce(move |_, a, b| f(a, b));

        if failed.load(Ordering::Relaxed) {
            return None;
        }

        result = match (result, wave_result) {
            (Some(a), Some(b)) => Some(f(a, b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };

        let len = data.iter().map(|x| x.len()).sum();
        utils::into_outer(&mut outer, len, &mut data);

        if outer.is_empty() {
            break;
        }
    }

    Some(result)
}
