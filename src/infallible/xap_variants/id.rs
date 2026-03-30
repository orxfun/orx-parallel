use crate::infallible::fun::map::FnMap;
use crate::infallible::size::One;
use crate::infallible::xap::Xap;
use crate::infallible::xap_variants::one_m::OneM;
use core::marker::PhantomData;

pub struct Id<I>(PhantomData<I>);

impl<I> Clone for Id<I> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<I> Copy for Id<I> {}

unsafe impl<I> Send for Id<I> {}

impl<I> Id<I> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<I> Xap for Id<I> {
    type I = I;

    type O = I;

    type Size = One;

    type Values = [I; 1];

    fn xap(&self, i: Self::I) -> Self::Values {
        [i]
    }

    // transformations

    type Map<Q, H>
        = OneM<Self, FnMap<Self::O, Q, H>>
    where
        H: Fn(Self::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q + Copy + Send,
    {
        OneM::new(self, FnMap::new(h))
    }
}
