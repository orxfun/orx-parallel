use crate::par::par_empty::Par;
use orx_concurrent_iter::*;
use std::ops::{Add, Range, Sub};

pub trait IntoPar {
    type ConIter: ConcurrentIter;

    fn into_par(self) -> Par<Self::ConIter>;
}

// array
impl<const N: usize, T: Send + Sync + Default> IntoPar for [T; N] {
    type ConIter = ConIterOfArray<N, T>;
    fn into_par(self) -> Par<Self::ConIter> {
        Par::new(self.into_con_iter())
    }
}
impl<const N: usize, T: Send + Sync + Default> IntoPar for ConIterOfArray<N, T> {
    type ConIter = ConIterOfArray<N, T>;
    fn into_par(self) -> Par<Self::ConIter> {
        Par::new(self)
    }
}

// iter
impl<T: Send + Sync, Iter> IntoPar for ConIterOfIter<T, Iter>
where
    Iter: Iterator<Item = T>,
{
    type ConIter = ConIterOfIter<T, Iter>;
    fn into_par(self) -> Par<Self::ConIter> {
        Par::new(self)
    }
}

// range
impl<Idx> IntoPar for Range<Idx>
where
    Idx: Send
        + Sync
        + Clone
        + Copy
        + From<usize>
        + Into<usize>
        + Add<Idx, Output = Idx>
        + Sub<Idx, Output = Idx>
        + Ord,
{
    type ConIter = ConIterOfRange<Idx>;
    fn into_par(self) -> Par<Self::ConIter> {
        Par::new(self.into_con_iter())
    }
}
impl<Idx> IntoPar for ConIterOfRange<Idx>
where
    Idx: Send
        + Sync
        + Clone
        + Copy
        + From<usize>
        + Into<usize>
        + Add<Idx, Output = Idx>
        + Sub<Idx, Output = Idx>
        + Ord,
{
    type ConIter = ConIterOfRange<Idx>;
    fn into_par(self) -> Par<Self::ConIter> {
        Par::new(self)
    }
}

// slice
impl<'a, T: Send + Sync> IntoPar for &'a [T] {
    type ConIter = ConIterOfSlice<'a, T>;
    fn into_par(self) -> Par<Self::ConIter> {
        Par::new(self.into_con_iter())
    }
}

impl<'a, T: Send + Sync> IntoPar for ConIterOfSlice<'a, T> {
    type ConIter = ConIterOfSlice<'a, T>;
    fn into_par(self) -> Par<Self::ConIter> {
        Par::new(self)
    }
}

// cloned slice

impl<'a, T: Send + Sync + Clone> IntoPar for ClonedConIterOfSlice<'a, T> {
    type ConIter = ClonedConIterOfSlice<'a, T>;
    fn into_par(self) -> Par<Self::ConIter> {
        Par::new(self)
    }
}

// vec
impl<T: Send + Sync + Default> IntoPar for Vec<T> {
    type ConIter = ConIterOfVec<T>;
    fn into_par(self) -> Par<Self::ConIter> {
        Par::new(self.into_con_iter())
    }
}
impl<T: Send + Sync + Default> IntoPar for ConIterOfVec<T> {
    type ConIter = ConIterOfVec<T>;
    fn into_par(self) -> Par<Self::ConIter> {
        Par::new(self)
    }
}
