use super::data::MapCollectData;
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;

fn task<I, O, Map, P>(
    x: MapCollectData<I, O, Map, P>,
    chunk_size: usize,
    bag: &ConcurrentOrderedBag<O, P>,
) where
    I: ConcurrentIter,
    O: Send + Sync,
    Map: Fn(I::Item) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    let iter = x.iter.enumerated();
    match chunk_size {
        0 | 1 => collect_within_thread(iter.item_puller(), x.map, bag),
        c => collect_within_thread(iter.chunk_puller(c).flattened(), x.map, bag),
    }
}

fn collect_within_thread<T, I, O, Map, P>(puller: I, map: Map, bag: &ConcurrentOrderedBag<O, P>)
where
    I: Iterator<Item = (usize, T)>,
    O: Send + Sync,
    Map: Fn(T) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    for (i, value) in puller {
        unsafe { bag.set_value(i, map(value)) };
    }
}
