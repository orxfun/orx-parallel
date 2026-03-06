use crate::xap::stopper::Stopper;
use crate::xap::xap_trait::{Elem, Xap};
use core::marker::PhantomData;

pub struct Faker<I, O, S: Stopper> {
    p: PhantomData<(I, O, S)>,
}

impl<I, O, S: Stopper> Xap for Faker<I, O, S> {
    type I = I;

    type O = O;

    type S = S;

    type Values = [Elem<Self>; 1];

    fn xap(&self, _: Self::I) -> Self::Values {
        todo!()
    }

    type Map<Q, G>
        = Faker<I, Q, S>
    where
        G: Fn(Self::O) -> Q;

    type Inspect<G>
        = Faker<I, O, S>
    where
        G: Fn(&Self::O);

    type Filter<G>
        = Faker<I, O, S>
    where
        G: Fn(&Self::O) -> bool;

    type FilterMap<Q, G>
        = Faker<I, Q, S>
    where
        G: Fn(Self::O) -> Option<Q>;

    type FlatMap<V, G>
        = Faker<I, V::Item, S>
    where
        V: IntoIterator,
        G: Fn(Self::O) -> V;
}
