use crate::xap::faker::Faker;
use crate::xap::xap_implementors::f::F;
use crate::xap::xap_trait::{IterOf, Xap};

pub struct M<X: Xap, O, G: Fn(X::O) -> O> {
    x: X,
    g: G,
}

impl<X: Xap, O, G: Fn(X::O) -> O> M<X, O, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap, O, G: Fn(X::O) -> O> Xap for M<X, O, G> {
    type I = X::I;

    type O = O;

    type Values<'i>
        = core::iter::Map<IterOf<'i, X>, &'i G>
    where
        Self: 'i;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values<'_> {
        self.x.xap(i).into_iter().map(&self.g)
    }

    // transformations

    type Map<Q, H>
        = Faker<Self::I, Q>
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
