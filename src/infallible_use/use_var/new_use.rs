use crate::infallible_use::{UseClone, UseFun, use_var::use_dyn_vec::UseDynVec};

pub struct Use;

impl Use {
    pub fn fun<T, F>(f: F) -> UseFun<T, F>
    where
        F: Fn(usize) -> T + Sync,
    {
        UseFun::new(f)
    }

    pub fn clone<T: Clone + Send>(value: T) -> UseClone<T> {
        UseClone::new(value)
    }

    pub fn dyn_vec<T, F: Fn(usize) -> T>(init: F) -> UseDynVec<T, F> {
        UseDynVec::new(init)
    }
}
