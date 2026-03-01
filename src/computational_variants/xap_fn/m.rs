use crate::{
    computational_variants::xap_fn::{ff::FF, filter::Filter, xap::XapFn},
    generic_values::Values,
};
use core::marker::PhantomData;

pub struct M<I, O, X>
where
    X: Fn(I) -> O,
{
    f: X,
    p: PhantomData<I>,
}

impl<I, O, X> M<I, O, X>
where
    X: Fn(I) -> O,
{
    pub fn new(f: X) -> Self {
        let p = PhantomData;
        Self { f, p }
    }
}
