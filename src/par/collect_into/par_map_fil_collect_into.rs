use crate::ParMapFilter;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;
use orx_pinned_vec::PinnedVec;
use orx_split_vec::{Growth, SplitVec};

pub(crate) fn merge_bag_and_positions<T, P, Q, Push>(mut bag: P, positions: &Q, push: &mut Push)
where
    T: Default,
    P: PinnedVec<T>,
    Q: PinnedVec<usize>,
    Push: FnMut(T),
{
    // TODO: inefficient!
    for &x in positions.iter().filter(|x| **x < usize::MAX) {
        let mut value = Default::default();
        let b = bag.get_mut(x).expect("is-some");
        std::mem::swap(b, &mut value);
        push(value);
    }
}

pub trait ParMapFilterCollectInto<O: Send + Sync + Default> {
    fn map_filter_into<I, M, F>(self, par: ParMapFilter<I, O, M, F>) -> Self
    where
        I: ConcurrentIter,
        M: Fn(I::Item) -> O + Send + Sync + Clone,
        F: Fn(&O) -> bool + Send + Sync + Clone;
}

impl<O: Send + Sync + Default> ParMapFilterCollectInto<O> for Vec<O> {
    fn map_filter_into<I, M, F>(mut self, par: ParMapFilter<I, O, M, F>) -> Self
    where
        I: ConcurrentIter,
        M: Fn(I::Item) -> O + Send + Sync + Clone,
        F: Fn(&O) -> bool + Send + Sync + Clone,
    {
        par.collect_bag(|x| self.push(x));
        self
    }
}

impl<O: Send + Sync + Default> ParMapFilterCollectInto<O> for FixedVec<O> {
    fn map_filter_into<I, M, F>(self, par: ParMapFilter<I, O, M, F>) -> Self
    where
        I: ConcurrentIter,
        M: Fn(I::Item) -> O + Send + Sync + Clone,
        F: Fn(&O) -> bool + Send + Sync + Clone,
    {
        let mut vec: Vec<_> = self.into();

        if let Some(iter_len) = par.iter_len() {
            vec.reserve(iter_len);
        }

        par.collect_bag(|x| vec.push(x));
        vec.into()
    }
}

impl<O: Send + Sync + Default, G: Growth> ParMapFilterCollectInto<O> for SplitVec<O, G> {
    fn map_filter_into<I, M, F>(mut self, par: ParMapFilter<I, O, M, F>) -> Self
    where
        I: ConcurrentIter,
        M: Fn(I::Item) -> O + Send + Sync + Clone,
        F: Fn(&O) -> bool + Send + Sync + Clone,
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
