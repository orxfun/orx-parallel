use crate::result::recursive::utils;
use crate::result::recursive::xap_sync::XapSync;
use crate::{Par, ParDrain, ParThreadPool, ParUse, Params, infallible::Xap, runner::ParRunner};
use alloc::vec::Vec;

struct Local<I, O, E> {
    input: Vec<I>,
    output: Vec<O>,
    error: Option<E>,
}

impl<I, O, E> Local<I, O, E> {
    fn new() -> Self {
        Self {
            input: Default::default(),
            output: Default::default(),
            error: None,
        }
    }
}

pub fn collect_arb<R, C, M, E, X1, X2, I, Ex>(
    mut runner: R,
    params: Params,
    iter: C,
    x1: X1,
    x2: X2,
    extend: Ex,
) -> Result<Vec<Vec<X2::O>>, E>
where
    R: ParRunner,
    C: IntoIterator,
    C::Item: Clone + Send + Sync,
    X1: Xap<I = C::Item, O = Result<M, E>>,
    X2: Xap<I = M>,
    I: IntoIterator<Item = C::Item>,
    Ex: Fn(&C::Item) -> I + Send + Sync,
    X2::O: Send + Sync,
    M: Send + Sync,
    E: Send + Sync,
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
            if u.error.is_some() {
                return;
            }
            let i_clone = i.clone();
            match utils::split(&x1, i) {
                Ok(ms) => {
                    u.input.extend(extend(&i_clone));
                    for m in ms {
                        u.output.extend(x2.xap(m));
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

    Ok(data.into_iter().map(|x| x.output).collect())
}
