use super::data::MapCollectData;
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter, Enumerated};
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;

pub fn map_col<I, O, Map, P>(x: &MapCollectData<I, O, Map, P>, chunk_size: usize)
where
    I: ConcurrentIter<Enumerated>,
    O: Send + Sync,
    Map: Fn(I::Item) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    match chunk_size {
        0 | 1 => pull_into_bag(x.iter.item_puller(), &x.map, &x.bag),
        c => pull_into_bag(x.iter.chunk_puller(c).flattened(), &x.map, &x.bag),
    }
}

fn pull_into_bag<T, I, O, Map, P>(puller: I, map: &Map, bag: &ConcurrentOrderedBag<O, P>)
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
