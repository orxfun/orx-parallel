use crate::collectables::{
    Vec2, inf::ColIntoInf, inf_use::ColIntoInfUse, opt::ColIntoOpt, opt_use::ColIntoOptUse,
    res::ColIntoRes, res_use::ColIntoResUse,
};
use alloc::vec::Vec;
use orx_fixed_vec::FixedVec;
use orx_split_vec::{Doubling, Linear, Recursive, SplitVec};

pub trait ParCollectInto<T>:
    ColIntoInf<T>
    + ColIntoRes<T>
    + ColIntoOpt<T>
    + ColIntoInfUse<T>
    + ColIntoResUse<T>
    + ColIntoOptUse<T>
{
}

impl<T> ParCollectInto<T> for FixedVec<T> {}

impl<T> ParCollectInto<T> for SplitVec<T, Doubling> {}

impl<T> ParCollectInto<T> for SplitVec<T, Linear> {}

impl<T> ParCollectInto<T> for SplitVec<T, Recursive> {}

impl<T> ParCollectInto<T> for Vec<T> {}

impl<T> ParCollectInto<T> for Vec2<T> {}
