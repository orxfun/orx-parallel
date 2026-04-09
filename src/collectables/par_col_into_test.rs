use crate::ParCollectInto;
use alloc::vec::Vec;
use orx_fixed_vec::{FixedVec, PinnedVec};
use orx_split_vec::{GrowthWithConstantTimeAccess, SplitVec};

pub trait ParCollectIntoTest<T: Clone>: ParCollectInto<T> + Clone {
    fn expected(initial: &Self, iter: impl IntoIterator<Item = T>) -> Self;
}

impl<T: Clone> ParCollectIntoTest<T> for FixedVec<T> {
    fn expected(initial: &Self, iter: impl IntoIterator<Item = T>) -> Self {
        let mut vec = initial.clone();
        for i in iter {
            vec.push(i);
        }
        vec
    }
}

impl<T: Clone, G: GrowthWithConstantTimeAccess> ParCollectIntoTest<T> for SplitVec<T, G> {
    fn expected(initial: &Self, iter: impl IntoIterator<Item = T>) -> Self {
        let mut vec = initial.clone();
        vec.extend(iter);
        vec
    }
}

impl<T: Clone> ParCollectIntoTest<T> for Vec<T> {
    fn expected(initial: &Self, iter: impl IntoIterator<Item = T>) -> Self {
        let mut vec = initial.clone();
        vec.extend(iter);
        vec
    }
}
