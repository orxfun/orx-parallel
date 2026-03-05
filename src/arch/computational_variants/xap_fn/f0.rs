use crate::{
    computational_variants::xap_fn::{f::F, ff::FF, filter::Filter, xap::XapFn},
    generic_values::Values,
};
use core::marker::PhantomData;

pub struct F0<I> {
    p: PhantomData<I>,
}

impl<I> F0<I> {
    pub const fn new() -> Self {
        let p = PhantomData;
        Self { p }
    }
}

impl<I> Filter<I> for F0<I> {
    #[inline(always)]
    fn filter(&self, _: &I) -> bool {
        true
    }

    type Compose<Y>
        = F<I, Y>
    where
        Y: Fn(&I) -> bool;

    fn compose<Y: Fn(&I) -> bool>(self, y: Y) -> Self::Compose<Y> {
        F::new(y)
    }
}
