use super::data::MapCollectData;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::IntoConcurrentPinnedVec;

fn map_collect_sequentially<I, O, Map, P>(data: MapCollectData<I, O, Map, P>) -> P
where
    I: ConcurrentIter,
    O: Send + Sync,
    Map: Fn(I::Item) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    // # SAFETY: collected is just wrapped as a concurrent-ordered-bag and is not mutated,
    // hence it is safe to convert it back to the underlying pinned vector.
    let mut vec = unsafe { data.collected.into_inner().unwrap_only_if_counts_match() };

    for x in data.iter.into_seq_iter().map(data.map) {
        vec.push(x);
    }

    vec
}
