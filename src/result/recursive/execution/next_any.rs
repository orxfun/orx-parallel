use crate::result::recursive::utils;
use crate::result::recursive::xap_sync::XapSync;
use crate::{Par, ParDrain, ParThreadPool, ParUse, Params, infallible::Xap, runner::ParRunner};
use alloc::vec::Vec;

struct Local<I, E> {
    input: Vec<I>,
    error: Option<E>,
}

impl<I, E> Local<I, E> {
    fn new() -> Self {
        Self {
            input: Default::default(),
            error: None,
        }
    }
}

pub fn next_any<R, C, M, E, X1, X2, I, Ex>(
    mut runner: R,
    params: Params,
    iter: C,
    x1: X1,
    x2: X2,
    extend: Ex,
) -> Result<Option<X2::O>, E>
where
    R: ParRunner,
    C: IntoIterator,
    C::Item: Clone + Send + Sync,
    X1: Xap<I = C::Item, O = Result<M, E>>,
    X2: Xap<I = M>,
    I: IntoIterator<Item = C::Item>,
    Ex: Fn(&C::Item) -> I + Send + Sync,
    X2::O: Send,
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

        let found = par
            .flat_map(|u, i| {
                if u.error.is_some() {
                    return Vec::new();
                }
                let i_clone = i.clone();
                match utils::split(&x1, i) {
                    Ok(ms) => {
                        u.input.extend(extend(&i_clone));
                        ms.into_iter().flat_map(|m| x2.xap(m)).collect::<Vec<_>>()
                    }
                    Err(e) => {
                        u.error = Some(e);
                        Vec::new()
                    }
                }
            })
            .first();

        if let Some(e) = data.iter_mut().find_map(|d| d.error.take()) {
            return Err(e);
        }

        if found.is_some() {
            return Ok(found);
        }

        let len = data.iter().map(|x| x.input.len()).sum();
        utils::into_outer(&mut outer, len, data.iter_mut().map(|x| &mut x.input));

        if outer.is_empty() {
            break;
        }
    }

    Ok(None)
}
