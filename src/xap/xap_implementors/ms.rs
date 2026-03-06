use crate::xap::faker::Faker;
use crate::xap::stopper::NeverStop;
use crate::xap::xap_trait::{Elem, Xap};
use core::marker::PhantomData;

pub struct Ms<I, O, F: Fn(I) -> O> {
    f: F,
    p: PhantomData<(I, O)>,
}

impl<I, O, F: Fn(I) -> O> Ms<I, O, F> {
    pub fn new(f: F) -> Self {
        let p = PhantomData;
        Self { f, p }
    }
}

impl<I, O, F: Fn(I) -> O> Xap for Ms<I, O, F> {
    type I = I;

    type O = O;

    type S = NeverStop;

    type Values = [Elem<Self>; 1];

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values {
        [Ok((self.f)(i))]
    }

    // transformations

    type Map<Q, G>
        = Faker<Self::I, Q, Self::S>
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
