use crate::xap::fun::map::r#fn::MapFn;
use core::marker::PhantomData;

// map

pub struct MWr<I, O, F: Fn(I) -> O>(F, PhantomData<(I, O)>);

impl<I, O, F: Fn(I) -> O> MWr<I, O, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<I, O, F: Fn(I) -> O> MapFn for MWr<I, O, F> {
    type I = I;

    type O = O;

    #[inline(always)]
    fn map(&self, i: Self::I) -> Self::O {
        (self.0)(i)
    }
}

// inspect

pub struct InsWr<I, F: Fn(&I)>(F, PhantomData<I>);

impl<I, F: Fn(&I)> InsWr<I, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<I, F: Fn(&I)> MapFn for InsWr<I, F> {
    type I = I;

    type O = I;

    #[inline(always)]
    fn map(&self, i: Self::I) -> Self::O {
        (self.0)(&i);
        i
    }
}
