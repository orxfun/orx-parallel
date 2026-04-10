use crate::infallible_use::{Use, xap::XapUse};
use crate::runner::ParRunner;
use alloc::vec::Vec;
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn collect_arb<Q, U, I, X>(u: &U, th_idx: usize, state: &Q::State, iter: &I, x: X) -> Vec<X::O>
where
    Q: ParRunner,
    U: Use,
    I: ConcurrentIter,
    X: XapUse<U = U::Item, I = I::Item>,
{
    let mut collected = Vec::new();
    let vec = &mut collected;

    let mut u = u.create(th_idx);
    let u = &mut u as *mut U::Item;

    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller();

    loop {
        let chunk_size = Q::next_chunk_size(state, iter.try_get_len());
        let chunk_state = Q::begin_chunk(th_idx, chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some(i) => vec.extend(x.xap_use(u, i)),
                None if iter.is_completed_when_none_returned() => break,
                None => {}
            },
            c => {
                if c > chunk_puller.chunk_size() {
                    chunk_puller = iter.chunk_puller(c);
                }

                match chunk_puller.pull() {
                    Some(chunk) => vec.extend(chunk.flat_map(|i| x.xap_use(u, i))),
                    None if iter.is_completed_when_none_returned() => break,
                    None => {}
                }
            }
        }

        Q::complete_chunk(state, chunk_state);
    }

    collected
}
