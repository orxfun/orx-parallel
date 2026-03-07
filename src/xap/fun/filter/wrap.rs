use crate::xap::fun::filter::r#fn::FilterFn;
use core::marker::PhantomData;

pub struct FilWrap<I, F: Fn(&I) -> bool>(F, PhantomData<I>);

impl<I, F: Fn(&I) -> bool> FilWrap<I, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<I, F: Fn(&I) -> bool> FilterFn for FilWrap<I, F> {
    type I = I;

    #[inline(always)]
    fn filter(&self, i: &Self::I) -> bool {
        (self.0)(i)
    }
}
