use crate::result::recursive::utils;
use crate::result::recursive::xap_sync::XapSync;
use crate::{Par, ParDrain, ParThreadPool, ParUse, Params, infallible::Xap, runner::ParRunner};
use alloc::vec::Vec;

struct Local<I, B, E> {
    input: Vec<I>,
    fold: B,
    error: Option<E>,
}

impl<I, B, E> Local<I, B, E> {
    fn new(init: B) -> Self {
        Self {
            input: Default::default(),
            fold: init,
            error: None,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn fold<R, C, M, E, X1, X2, B, Id, F, I, Ex>(
    mut runner: R,
    params: Params,
    iter: C,
    x1: X1,
    x2: X2,
    extend: Ex,
    init: Id,
    f: F,
) -> Result<Vec<B>, E>
where
    R: ParRunner,
    C: IntoIterator,
    C::Item: Clone + Send + Sync,
    X1: Xap<I = C::Item, O = Result<M, E>>,
    X2: Xap<I = M>,
    I: IntoIterator<Item = C::Item>,
    Ex: Fn(&C::Item) -> I + Send + Sync,
    B: Send + Sync,
    Id: Fn() -> B + Sync,
    F: Fn(&mut B, X2::O) + Copy + Send + Sync,
    M: Send + Sync,
    E: Send + Sync,
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
            if u.error.is_some() {
                return;
            }
            let i_clone = i.clone();
            match utils::split(&x1, i) {
                Ok(ms) => {
                    u.input.extend(extend(&i_clone));
                    for m in ms {
                        for out in x2.xap(m) {
                            f(&mut u.fold, out);
                        }
                    }
                }
                Err(e) => u.error = Some(e),
            }
        });

        if let Some(e) = data.iter_mut().find_map(|d| d.error.take()) {
            return Err(e);
        }

        let len = data.iter().map(|x| x.input.len()).sum();
        utils::into_outer(&mut outer, len, data.iter_mut().map(|x| &mut x.input));

        if outer.is_empty() {
            break;
        }
    }

    Ok(data.into_iter().map(|x| x.fold).collect())
}
