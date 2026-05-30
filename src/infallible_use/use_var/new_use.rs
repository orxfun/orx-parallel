use crate::infallible_use::{UseClone, UseFun};

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
}
