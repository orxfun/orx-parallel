use crate::xap::faker::Faker;
use crate::xap::xap_implementors::f::F;
use crate::xap::xap_implementors::m::M;
use crate::xap::xap_trait::Xap;
use core::marker::PhantomData;

pub struct Id<I>(PhantomData<I>);

impl<I> Id<I> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<I> Xap for Id<I> {
    type I = I;

    type O = I;

    type Values<'i>
        = [I; 1]
    where
        Self: 'i;

    fn xap(&self, i: Self::I) -> Self::Values<'_> {
        [i]
    }

    // transformations

    type Map<Q, H>
        = M<Self, Q, H>
    where
        H: Fn(Self::O) -> Q;

    type Inspect<H>
        = Faker<Self::I, Self::O>
    where
        H: Fn(&Self::O);

    type Filter<H>
        = F<Self, H>
    where
        H: Fn(&Self::O) -> bool;

    type FilterMap<Q, H>
        = Faker<Self::I, Q>
    where
        H: Fn(Self::O) -> Option<Q>;

    type FlatMap<V, H>
        = Faker<Self::I, V::Item>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V;
}
