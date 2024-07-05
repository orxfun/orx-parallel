use crate::{
    fn_sync::FnSync,
    par::{
        par_filtermap_fil::ParFilterMapFilter, par_flatmap_fil::ParFlatMapFilter, par_map::ParMap,
        par_map_fil::ParMapFilter,
    },
    Fallible,
};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;
use orx_pinned_vec::PinnedVec;
use std::{fmt::Debug, mem::ManuallyDrop};

pub trait ParCollectIntoCore<O: Send + Sync + Debug> {
    type BridgePinnedVec: PinnedVec<O>;

    /// Performs the parallel map operation, collecting the results into this collection.
    fn map_into<I, M>(self, par_map: ParMap<I, O, M>) -> Self
    where
        I: ConcurrentIter,
        M: Fn(I::Item) -> O + FnSync;

    fn map_filter_into<I, M, F>(self, par: ParMapFilter<I, O, M, F>) -> Self
    where
        I: ConcurrentIter,
        M: Fn(I::Item) -> O + FnSync,
        F: Fn(&O) -> bool + FnSync;

    fn flatmap_filter_into<I, OI, M, F>(self, par: ParFlatMapFilter<I, O, OI, M, F>) -> Self
    where
        I: ConcurrentIter,
        OI: IntoIterator<Item = O>,
        M: Fn(I::Item) -> OI + Send + Sync,
        F: Fn(&O) -> bool + Send + Sync;

    fn filtermap_filter_into<I, FO, M, F>(self, par: ParFilterMapFilter<I, FO, O, M, F>) -> Self
    where
        I: ConcurrentIter,
        FO: Fallible<O> + Send + Sync + Debug,
        M: Fn(I::Item) -> FO + FnSync,
        F: Fn(&O) -> bool + FnSync;

    fn into_concurrent_bag(self) -> ConcurrentBag<O, Self::BridgePinnedVec>;

    fn from_concurrent_bag(bag: ConcurrentBag<O, Self::BridgePinnedVec>) -> Self;

    fn seq_extend<I: Iterator<Item = O>>(self, iter: I) -> Self
    where
        Self: Sized;
}

const ERR_SRC: &str = "is in bounds";
const ERR_DST: &str = "output has enough capacity";

pub fn merge_bag_and_positions<T, P, Q, O>(mut bag: P, positions: &Q, output: &mut O)
where
    T: Debug,
    P: PinnedVec<T>,
    Q: PinnedVec<usize>,
    O: PinnedVec<T> + Debug,
{
    match bag.is_empty() {
        true => {}
        false => {
            // assert!(output.capacity() >= bag.len());

            let mut des_idx = output.len();

            for &src_idx in positions.iter().filter(|x| **x < usize::MAX) {
                let source_ptr = unsafe { bag.get_ptr_mut(src_idx) }.expect(ERR_SRC);
                let destination_ptr = unsafe { output.get_ptr_mut(des_idx) }.expect(ERR_DST);

                unsafe { destination_ptr.write(source_ptr.read()) };

                des_idx += 1;
            }

            unsafe { output.set_len(des_idx) };
            let _ = ManuallyDrop::new(bag);
        }
    }
}
