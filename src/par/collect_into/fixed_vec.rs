use super::par_collect_into::ParCollectInto;
use orx_concurrent_bag::ConcurrentBag;
use orx_fixed_vec::FixedVec;

impl<O: Default + Send + Sync> ParCollectInto<O> for FixedVec<O> {
    type BridgePinnedVec = Self;

    fn map_into<I, M>(self, par_map: crate::ParMap<I, O, M>) -> Self
    where
        I: orx_concurrent_iter::ConcurrentIter,
        M: Fn(I::Item) -> O + Send + Sync + Clone,
    {
        let vec: Vec<_> = self.into();
        vec.map_into(par_map).into()
    }

    fn map_filter_into<I, M, F>(self, par: crate::ParMapFilter<I, O, M, F>) -> Self
    where
        I: orx_concurrent_iter::ConcurrentIter,
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

    fn fmap_filter_into<I, OI, M, F>(
        self,
        par: crate::par::par_fmap_fil::ParFMapFilter<I, O, OI, M, F>,
    ) -> Self
    where
        I: orx_concurrent_iter::ConcurrentIter,
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

    fn into_concurrent_bag(self) -> ConcurrentBag<O, Self::BridgePinnedVec> {
        self.into()
    }

    fn from_concurrent_bag(bag: ConcurrentBag<O, Self::BridgePinnedVec>) -> Self {
        bag.into_inner().into()
    }
}
