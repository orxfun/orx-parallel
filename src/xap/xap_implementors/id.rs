use crate::xap::faker::Faker;
use crate::xap::stopper::NeverStop;
use crate::xap::xap_implementors::ms::Ms;
use crate::xap::xap_trait::{Elem, Xap};
use core::marker::PhantomData;

pub struct Id<I>(PhantomData<I>);

impl<I> Id<I> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<I> Xap for Id<I> {
    type I = I;

    type O = I;

    type S = NeverStop;

    type Values = [Elem<Self>; 1];

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values {
        [Ok(i)]
    }

    // transformations

    type Map<Q, G>
        = Ms<I, Q, G>
    where
        G: Fn(Self::O) -> Q;

    type Inspect<G>
        = Faker<Self::I, Self::O, Self::S>
    where
        G: Fn(&Self::O);

    type Filter<G>
        = Faker<Self::I, Self::O, Self::S>
    where
        G: Fn(&Self::O) -> bool;

    type FilterMap<Q, G>
        = Faker<Self::I, Q, Self::S>
    where
        G: Fn(Self::O) -> Option<Q>;

    type FlatMap<V, G>
        = Faker<Self::I, V::Item, Self::S>
    where
        V: IntoIterator,
        G: Fn(Self::O) -> V;
}
