use crate::xap::fun::flat_map::fn_trait::FlatMap;
use core::marker::PhantomData;

pub struct FnFlatMap<I, O: IntoIterator, F: Fn(I) -> O>(F, PhantomData<(I, O)>);

impl<I, O: IntoIterator, F: Fn(I) -> O> FnFlatMap<I, O, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<I, O: IntoIterator, F: Fn(I) -> O> FlatMap for FnFlatMap<I, O, F> {
    type I = I;

    type O = O;

    #[inline(always)]
    fn flat_map(&self, i: Self::I) -> Self::O {
        (self.0)(i)
    }
}
