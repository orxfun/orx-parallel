use crate::par::par_fmap_fil::ParFMapFilter;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;
use orx_pinned_vec::PinnedVec;
use orx_split_vec::{Growth, SplitVec};
use std::fmt::Debug;

pub(crate) fn merge_bag_and_pos_len<T, P, Q, Push>(mut bag: P, pos_len: &Q, push: &mut Push)
where
    T: Default,
    P: PinnedVec<T>,
    Q: PinnedVec<(usize, usize)>,
    Push: FnMut(T),
{
    // TODO: inefficient! SPlitVec into_iter might solve it
    for &x in pos_len.iter().filter(|x| x.0 < usize::MAX && x.1 > 0) {
        let (begin_idx, len) = (x.0, x.1);
        for i in 0..len {
            let mut value = Default::default();
            let idx = begin_idx + i;
            let b = bag.get_mut(idx).expect("is-some");
            std::mem::swap(b, &mut value);
            push(value);
        }
    }
}

pub trait ParFMapFilterCollectInto<O: Send + Sync + Default> {
    fn fmap_filter_into<I, OI, M, F>(self, par: ParFMapFilter<I, O, OI, M, F>) -> Self
    where
        I: ConcurrentIter,
        OI: IntoIterator<Item = O>,
        M: Fn(I::Item) -> OI + Send + Sync,
        F: Fn(&O) -> bool + Send + Sync;
}

impl<O: Send + Sync + Default + Debug> ParFMapFilterCollectInto<O> for Vec<O> {
    fn fmap_filter_into<I, OI, M, F>(mut self, par: ParFMapFilter<I, O, OI, M, F>) -> Self
    where
        I: ConcurrentIter,
        OI: IntoIterator<Item = O>,
        M: Fn(I::Item) -> OI + Send + Sync,
        F: Fn(&O) -> bool + Send + Sync,
    {
        par.collect_bag(|x| self.push(x));
        self
    }
}

impl<O: Send + Sync + Default + Debug> ParFMapFilterCollectInto<O> for FixedVec<O> {
    fn fmap_filter_into<I, OI, M, F>(self, par: ParFMapFilter<I, O, OI, M, F>) -> Self
    where
        I: ConcurrentIter,
        OI: IntoIterator<Item = O>,
        M: Fn(I::Item) -> OI + Send + Sync,
        F: Fn(&O) -> bool + Send + Sync,
    {
        let mut vec: Vec<_> = self.into();

        if let Some(iter_len) = par.iter_len() {
            vec.reserve(iter_len);
        }

        par.collect_bag(|x| vec.push(x));
        vec.into()
    }
}

impl<O: Send + Sync + Default + Debug, G: Growth> ParFMapFilterCollectInto<O> for SplitVec<O, G> {
    fn fmap_filter_into<I, OI, M, F>(mut self, par: ParFMapFilter<I, O, OI, M, F>) -> Self
    where
        I: ConcurrentIter,
        OI: IntoIterator<Item = O>,
        M: Fn(I::Item) -> OI + Send + Sync,
        F: Fn(&O) -> bool + Send + Sync,
    {
        if let Some(iter_len) = par.iter_len() {
            let required_len = self.len() + iter_len;
            self.try_reserve_maximum_concurrent_capacity(required_len)
                .expect("Failed to reserve sufficient capacity");
        }

        par.collect_bag(|x| self.push(x));
        self
    }
}
