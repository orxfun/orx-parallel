use crate::ParCollectInto;
use alloc::vec::Vec;
use core::fmt::Debug;
use orx_fixed_vec::FixedVec;
use orx_pinned_vec::PinnedVec;
use orx_split_vec::{GrowthWithConstantTimeAccess, SplitVec};

pub trait ParCollectIntoTest<T: Clone + PartialEq + Debug>:
    ParCollectInto<T> + Clone + PartialEq + Debug
{
    fn expected(&self, iter: impl IntoIterator<Item = T>) -> Self;

    fn push_back(&mut self, value: T);
}

impl<T: Clone + PartialEq + Debug> ParCollectIntoTest<T> for FixedVec<T> {
    fn expected(&self, iter: impl IntoIterator<Item = T>) -> Self {
        let mut vec = self.clone().into_inner();
        vec.extend(iter);
        vec.into()
    }

    fn push_back(&mut self, value: T) {
        self.push(value);
    }
}

impl<T: Clone + PartialEq + Debug, G: GrowthWithConstantTimeAccess> ParCollectIntoTest<T>
    for SplitVec<T, G>
{
    fn expected(&self, iter: impl IntoIterator<Item = T>) -> Self {
        let mut vec = self.clone();
        vec.extend(iter);
        vec
    }

    fn push_back(&mut self, value: T) {
        self.push(value);
    }
}

impl<T: Clone + PartialEq + Debug> ParCollectIntoTest<T> for Vec<T> {
    fn expected(&self, iter: impl IntoIterator<Item = T>) -> Self {
        let mut vec = self.clone();
        vec.extend(iter);
        vec
    }

    fn push_back(&mut self, value: T) {
        self.push(value);
    }
}
