use crate::xap::faker::Faker;
use crate::xap::xap_trait::{IterOf, Xap};

pub struct M<X: Xap, O, F: Fn(X::O) -> O> {
    x: X,
    f: F,
}

impl<X: Xap, O, F: Fn(X::O) -> O> M<X, O, F> {
    pub fn new(x: X, f: F) -> Self {
        Self { x, f }
    }
}

impl<X: Xap, O, F: Fn(X::O) -> O> Xap for M<X, O, F> {
    type I = X::I;

    type O = O;

    type Values<'i>
        = core::iter::Map<IterOf<'i, X>, &'i F>
    where
        Self: 'i;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values<'_> {
        self.x.xap(i).into_iter().map(&self.f)
    }

    // transformations

    type Map<Q, G>
        = Faker<Self::I, Q>
    where
        G: Fn(Self::O) -> Q;

    type Inspect<G>
        = Faker<Self::I, Self::O>
    where
        G: Fn(&Self::O);

    type Filter<G>
        = Faker<Self::I, Self::O>
    where
        G: Fn(&Self::O) -> bool;

    type FilterMap<Q, G>
        = Faker<Self::I, Q>
    where
        G: Fn(Self::O) -> Option<Q>;

    type FlatMap<V, G>
        = Faker<Self::I, V::Item>
    where
        V: IntoIterator,
        G: Fn(Self::O) -> V;
}
