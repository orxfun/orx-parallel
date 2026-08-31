use crate::runner::ParRunner;
use crate::{collectables::Collectable, infallible_use::xap::XapUse};
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn collect_arb<Q, U, I, X, D>(u: &mut U, th_idx: usize, state: &Q::State, iter: &I, x: X) -> D
where
    Q: ParRunner,
    I: ConcurrentIter,
    X: XapUse<U = U, I = I::Item>,
    D: Collectable<X::O>,
{
    let mut collected = D::col_empty();
    let vec = &mut collected;

    let mut chunk_puller = iter.chunk_puller_by(0, th_idx);

    loop {
        let chunk_size = Q::next_chunk_size(state, iter.size_hint());
        let chunk_state = Q::begin_chunk(th_idx, chunk_size);

        match chunk_size {
            0 | 1 => match iter.next_by(th_idx) {
                Some(i) => vec.col_extend(x.xap_use(u, i)),
                None if iter.is_completed_when_none_returned() => break,
                None => {}
            },
            c => {
                chunk_puller.resize_for_chunk_size(c);

                match chunk_puller.pull() {
                    Some(chunk) => vec.col_extend(chunk.flat_map(|i| x.xap_use(u, i))),
                    None if iter.is_completed_when_none_returned() => break,
                    None => {}
                }
            }
        }

        Q::complete_chunk(state, chunk_state);
    }

    collected
}
