use crate::xap::fun::filter_map::r#fn::FilterMapFn;
use core::marker::PhantomData;

pub struct FilMWr<I, O, F: Fn(I) -> Option<O>>(F, PhantomData<(I, O)>);

impl<I, O, F: Fn(I) -> Option<O>> FilMWr<I, O, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<I, O, F: Fn(I) -> Option<O>> FilterMapFn for FilMWr<I, O, F> {
    type I = I;

    type O = O;

    #[inline(always)]
    fn filter_map(&self, i: Self::I) -> Option<Self::O> {
        (self.0)(i)
    }
}
