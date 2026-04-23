use crate::infallible_use::{Use, UseClone, UseFun};
use alloc::boxed::Box;

pub struct UseBox<'a, U> {
    u: Box<dyn Use<Item = U> + 'a>,
}

impl<'a, U: 'a> UseBox<'a, U> {
    pub fn fun<F>(f: F) -> Self
    where
        F: Fn(usize) -> U + Sync + 'a,
    {
        let use_fun = UseFun::new(f);
        let u: Box<dyn Use<Item = U> + 'a> = Box::new(use_fun);
        Self { u }
    }

    pub fn clone(value: U) -> Self
    where
        U: Clone + Send,
    {
        let use_clone = UseClone::new(value);
        let u: Box<dyn Use<Item = U> + 'a> = Box::new(use_clone);
        Self { u }
    }
}
