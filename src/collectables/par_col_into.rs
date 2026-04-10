use crate::collectables::{inf::ColIntoInf, opt::ColIntoOpt, res::ColIntoRes};
use alloc::vec::Vec;
use orx_fixed_vec::FixedVec;
use orx_split_vec::{GrowthWithConstantTimeAccess, SplitVec};

pub trait ParCollectInto<T>: ColIntoInf<T> + ColIntoRes<T> + ColIntoOpt<T> {}

impl<T> ParCollectInto<T> for FixedVec<T> {}

impl<T, G: GrowthWithConstantTimeAccess> ParCollectInto<T> for SplitVec<T, G> {}

impl<T> ParCollectInto<T> for Vec<T> {}
