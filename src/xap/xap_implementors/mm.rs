use crate::xap::faker::Faker;
use crate::xap::stopper::NeverStop;
use crate::xap::xap_trait::{Elem, Xap};
use core::marker::PhantomData;

pub struct Mm<X: Xap, O, F: Fn(X::O) -> O> {
    x: X,
    f: F,
}

impl<X: Xap, O, F: Fn(X::O) -> O> Mm<X, O, F> {
    pub fn new(x: X, f: F) -> Self {
        Self { x, f }
    }
}

impl<X: Xap, O, F: Fn(X::O) -> O> Xap for Mm<X, O, F> {
    type I = X::I;

    type O = O;

    type S = X::S;

    type Values = [Elem<Self>; 1]; // todo!

    fn xap(&self, i: Self::I) -> Self::Values {
        let x = self.x.xap(i);
        let mut y = x.into_iter();
        let z = y.next().unwrap();
        todo!()
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
