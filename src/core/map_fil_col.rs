use std::cmp::Ordering;

use super::params::RunParams;
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::PinnedVec;

pub fn map_fil_col<I, Out, Map, Fil, P, Q>(
    params: RunParams,
    iter: I,
    map: Map,
    filter: Fil,
    collected: ConcurrentBag<Out, P>,
    positions: ConcurrentOrderedBag<usize, Q>,
) -> (P, Q)
where
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    Fil: Fn(&Out) -> bool + Send + Sync,
    P: PinnedVec<Out>,
    Q: PinnedVec<usize>,
{
    let (num_threads, chunk_size) = (params.num_threads, params.chunk_size);
    let offset = collected.len();

    std::thread::scope(|s| {
        for _ in 0..num_threads {
            s.spawn(|| match chunk_size {
                1 => {
                    while let Some(x) = iter.next_id_and_value() {
                        let value = map(x.value);
                        let position = match filter(&value) {
                            true => collected.push(value),
                            false => usize::MAX,
                        };
                        let idx = offset + x.idx;
                        unsafe { positions.set_value(idx, position) };
                    }
                }
                _ => {
                    let mut buffered = iter.buffered_iter(chunk_size);
                    let mut local = vec![0usize; chunk_size];
                    while let Some(chunk) = buffered.next() {
                        let count = chunk.values.len();
                        let begin_idx = offset + chunk.begin_idx;

                        for (i, value) in chunk.values.map(&map).enumerate() {
                            let position = match filter(&value) {
                                true => collected.push(value),
                                false => usize::MAX,
                            };
                            local[i] = position;
                        }

                        unsafe {
                            match count.cmp(&chunk_size) {
                                Ordering::Less => positions
                                    .set_values(begin_idx, local.iter().take(count).cloned()),
                                _ => positions.set_values(begin_idx, local.iter().cloned()),
                            }
                        };
                    }
                }
            });
        }
    });

    (collected.into_inner(), unsafe {
        positions.into_inner().unwrap_only_if_counts_match()
    })
}
