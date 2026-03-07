use crate::xap::fun::filter::r#fn::FilterFn;
use core::marker::PhantomData;

pub struct FWr<I, F: Fn(&I) -> bool>(F, PhantomData<I>);

impl<I, F: Fn(&I) -> bool> FWr<I, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<I, F: Fn(&I) -> bool> FilterFn for FWr<I, F> {
    type I = I;

    #[inline(always)]
    fn filter(&self, i: &Self::I) -> bool {
        (self.0)(i)
    }
}
