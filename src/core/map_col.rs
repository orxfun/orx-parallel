use super::params::RunParams;
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::PinnedVec;

pub fn map_col<I, Out, Map, P>(
    params: RunParams,
    iter: I,
    map: Map,
    collected: ConcurrentOrderedBag<Out, P>,
) -> P
where
    I: ConcurrentIter,
    Out: Send + Sync,
    Map: Fn(I::Item) -> Out + Send + Sync,
    P: PinnedVec<Out>,
{
    let (num_threads, chunk_size) = (params.num_threads, params.chunk_size);
    let offset = collected.len();

    std::thread::scope(|s| {
        for _ in 0..num_threads {
            s.spawn(|| match chunk_size {
                1 => {
                    iter.ids_and_values()
                        .map(|(idx, value)| (offset + idx, map(value)))
                        .for_each(|(idx, value)| unsafe { collected.set_value(idx, value) });
                }
                _ => {
                    let mut buffered = iter.buffered_iter(chunk_size);
                    while let Some(chunk) = buffered.next() {
                        let begin_idx = offset + chunk.begin_idx;
                        unsafe { collected.set_values(begin_idx, chunk.values.map(&map)) };
                    }
                }
            });
        }
    });

    unsafe { collected.into_inner().unwrap_only_if_counts_match() }
}
