use crate::xap::fun::filter::fn_trait::Filter;
use core::marker::PhantomData;

pub struct FnFil<I, F: Fn(&I) -> bool>(F, PhantomData<I>);

impl<I, F: Fn(&I) -> bool> FnFil<I, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<I, F: Fn(&I) -> bool> Filter for FnFil<I, F> {
    type I = I;

    #[inline(always)]
    fn filter(&self, i: &Self::I) -> bool {
        (self.0)(i)
    }
}
