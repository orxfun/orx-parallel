use crate::infallible_use::Use;
use alloc::boxed::Box;

pub enum Using<T> {
    Fun(Box<dyn Fn(usize) -> T>),
    Cloning(T),
}

impl<T> Using<T> {
    pub fn fun(f: Box<dyn Fn(usize) -> T>) -> Self {
        Self::Fun(f)
    }

    pub fn cloning(value: T) -> Self {
        Self::Cloning(value)
    }

    // pub fn create(&self, thread_idx: usize) -> T {
    //     match self {
    //         Self::Fun(f) => f(thread_idx),
    //         Self::Cloning(x) => x.clone(),
    //     }
    // }
}

pub struct Abc<U>(Box<dyn Use<Item = U>>);

impl<U> Use for Abc<U> {
    type Item = U;

    #[inline]
    fn create(&self, thread_idx: usize) -> Self::Item {
        self.0.create(thread_idx)
    }
}
