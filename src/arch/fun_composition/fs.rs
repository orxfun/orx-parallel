use crate::fun_composition::xap::Xap;
use core::marker::PhantomData;

pub struct Fs<I, F: Fn(&I) -> bool> {
    f: F,
    p: PhantomData<I>,
}

impl<I, F: Fn(&I) -> bool> Fs<I, F> {
    pub fn new(f: F) -> Self {
        let p = PhantomData;
        Self { f, p }
    }
}
