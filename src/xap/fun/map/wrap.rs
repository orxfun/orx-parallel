use crate::xap::fun::map::r#fn::MapFn;
use core::marker::PhantomData;

pub struct MapWrap<I, O, F: Fn(I) -> O>(F, PhantomData<(I, O)>);

impl<I, O, F: Fn(I) -> O> MapWrap<I, O, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<I, O, F: Fn(I) -> O> MapFn for MapWrap<I, O, F> {
    type I = I;

    type O = O;

    #[inline(always)]
    fn map(&self, i: Self::I) -> Self::O {
        (self.0)(i)
    }
}
