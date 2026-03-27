use crate::execution::thread::thread_comp::ThreadComp;
use crate::xap::Xap;
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};
use orx_pinned_vec::IntoConcurrentPinnedVec;

pub fn collect_arbitrary<Q, I, X, P>(
    exe: &mut Q,
    state: &Q::SharedState,
    iter: &I,
    x: X,
    bag: &ConcurrentBag<X::O, P>,
) where
    Q: ThreadComp,
    I: ConcurrentIter,
    X: Xap<I = I::Item>,
    P: IntoConcurrentPinnedVec<X::O>,
    X::O: Send,
{
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller();

    loop {
        let chunk_size = exe.next_chunk_size(state, iter);
        exe.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some(i) => {
                    // TODO: bag.extend when we know exact size
                    for val in x.xap(i) {
                        bag.push(val);
                    }
                }
                None if iter.is_completed_when_none_returned() => break,
                None => {}
            },
            c => {
                if c > chunk_puller.chunk_size() {
                    chunk_puller = iter.chunk_puller(c);
                }

                match chunk_puller.pull() {
                    Some(chunk) => {
                        for val in chunk.flat_map(|i| x.xap(i)) {
                            bag.push(val);
                        }
                    }
                    None if iter.is_completed_when_none_returned() => break,
                    None => {}
                }
            }
        }
        exe.complete_chunk(state, chunk_size);
    }

    exe.complete_task(state);
}
