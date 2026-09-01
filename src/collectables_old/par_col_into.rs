use crate::collectables_old::{
    Collectable, Vec2, inf::ColIntoInf, inf_use::ColIntoInfUse, opt::ColIntoOpt,
    opt_use::ColIntoOptUse, res::ColIntoRes, res_use::ColIntoResUse,
};
use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use orx_fixed_vec::FixedVec;
use orx_split_vec::{Doubling, Linear, Recursive, SplitVec};

/// A collection that can receive items from the `.collect()` method of a parallel iterator.
///
/// Implemented for [`Vec`], [`Vec2`] (simply `Vec<Vec<_>>`), [`FixedVec`], and [`SplitVec`] with
/// [`Doubling`], [`Linear`], or [`Recursive`] growth.
pub trait ParCollectInto<T>:
    ColIntoInf<T>
    + ColIntoRes<T>
    + ColIntoOpt<T>
    + ColIntoInfUse<T>
    + ColIntoResUse<T>
    + ColIntoOptUse<T>
    + Collectable<T>
{
}

impl<T: Send> ParCollectInto<T> for FixedVec<T> {}

impl<T: Send> ParCollectInto<T> for SplitVec<T, Doubling> {}

impl<T: Send> ParCollectInto<T> for SplitVec<T, Linear> {}

impl<T: Send> ParCollectInto<T> for SplitVec<T, Recursive> {}

impl<T: Send> ParCollectInto<T> for Vec<T> {}

impl<T: Send> ParCollectInto<T> for Vec2<T> {}

// impl<T: Send + Ord> ParCollectInto<T> for BTreeSet<T> {}
