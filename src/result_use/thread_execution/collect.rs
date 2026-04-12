use crate::infallible_use::Use;
use crate::result_use::SizePairUseRes;
use crate::results::ValsAndIdx;
use crate::{infallible_use::XapUse, runner::ParRunner};
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn collect<Q, U, I, M, E, X1, X2, S>(
    _: S,
    u: &U,
    th_idx: usize,
    state: &Q::State,
    iter: &I,
    x1: X1,
    x2: X2,
) -> Result<ValsAndIdx<X2::O>, E>
where
    Q: ParRunner,
    U: Use,
    I: ConcurrentIter,
    X1: XapUse<U = U::Item, I = I::Item, O = Result<M, E>>,
    X2: XapUse<U = U::Item, I = M>,
    S: SizePairUseRes<S1 = X1::Size, S2 = X2::Size>,
{
    let mut collected = ValsAndIdx::new();
    let out = &mut collected;

    let mut u = u.create(th_idx);
    let u = &mut u as *mut U::Item;

    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller_with_idx();

    loop {
        let chunk_size = Q::next_chunk_size(state, iter.try_get_len());
        let chunk_state = Q::begin_chunk(th_idx, chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some((idx, i)) => {
                    let result = out.extend_res(idx, S::xap_use_res(u, x1, x2, i));
                    if let Some(e) = result {
                        Q::broadcast_stop(iter, state, chunk_state);
                        return Err(e);
                    }
                }
                None if iter.is_completed_when_none_returned() => break,
                None => {}
            },
            c => {
                if c > chunk_puller.chunk_size() {
                    chunk_puller = iter.chunk_puller(c);
                }

                match chunk_puller.pull_with_idx() {
                    Some((idx, chunk)) => {
                        let values = chunk.flat_map(|i| S::xap_use_res(u, x1, x2, i));
                        let result = out.extend_res(idx, values);
                        if let Some(e) = result {
                            Q::broadcast_stop(iter, state, chunk_state);
                            return Err(e);
                        }
                    }
                    None if iter.is_completed_when_none_returned() => break,
                    None => {}
                }
            }
        }

        Q::complete_chunk(state, chunk_state);
    }

    Ok(collected)
}
