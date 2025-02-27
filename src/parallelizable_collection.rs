use orx_concurrent_iter::ConcurrentIter;

pub trait ParallelizableCollection {
    type ParItem;

    type ParIter: ConcurrentIter<Item = Self::ParItem>;
    //
}
