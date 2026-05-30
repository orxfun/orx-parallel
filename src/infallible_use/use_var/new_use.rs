use crate::infallible_use::{UseClone, UseFun, UseSlice, UseVec};

pub struct Use;

impl Use {
    pub fn fun<T, F>(f: F) -> UseFun<T, F>
    where
        F: Fn(usize) -> T + Sync,
    {
        UseFun::new(f)
    }

    pub fn vec<T: Send, F: Fn(usize) -> T + Sync>(init: F) -> UseVec<T, F> {
        UseVec::new(init)
    }

    pub fn slice<'a, T: 'a>(slice: &'a mut [T]) -> UseSlice<'a, T> {
        UseSlice::new(slice)
    }
}
