use crate::option::recursive::utils;
use crate::option::recursive::xap_sync::XapSync;
use crate::{Par, ParDrain, ParThreadPool, ParUse, Params, infallible::Xap, runner::ParRunner};
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};

pub fn next_any<R, C, M, X1, X2, I, E>(
    mut runner: R,
    params: Params,
    iter: C,
    x1: X1,
    x2: X2,
    extend: E,
) -> Option<Option<X2::O>>
where
    R: ParRunner,
    C: IntoIterator,
    C::Item: Clone + Send + Sync,
    X1: Xap<I = C::Item, O = Option<M>>,
    X2: Xap<I = M>,
    I: IntoIterator<Item = C::Item>,
    E: Fn(&C::Item) -> I + Send + Sync,
    X2::O: Send,
    M: Send + Sync,
{
    let x1 = XapSync::new(x1);
    let x2 = XapSync::new(x2);
    let max_threads: usize = runner.pool().max_num_threads().into();

    let mut data: Vec<_> = (0..max_threads).map(|_| Vec::<C::Item>::new()).collect();
    let mut outer: Vec<_> = iter.into_iter().collect();
    let failed = AtomicBool::new(false);

    loop {
        let par = outer.par_drain(..).runner(&mut runner);
        let par = params.apply(par).use_slice(&mut data);

        let found = par
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
            .first();

        if failed.load(Ordering::Relaxed) {
            return None;
        }

        if found.is_some() {
            return Some(found);
        }

        let len = data.iter().map(|x| x.len()).sum();
        utils::into_outer(&mut outer, len, &mut data);

        if outer.is_empty() {
            break;
        }
    }

    Some(None)
}
