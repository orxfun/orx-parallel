use alloc::vec::Vec;
use orx_fixed_vec::FixedVec;
use orx_split_vec::{GrowthWithConstantTimeAccess, SplitVec};
use std::string::{String, ToString};

pub fn inputs(n: usize) -> Vec<String> {
    (0..n).map(|x| x.to_string()).collect()
}

pub enum ColInto<T, G: GrowthWithConstantTimeAccess> {
    Vec(Vec<T>),
    FixedVec(FixedVec<T>),
    SplitVec(SplitVec<T, G>),
}
