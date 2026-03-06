use crate::xap::faker::Faker;
use crate::xap::xap_implementors::m::M;
use crate::xap::xap_implementors::xap_iters::IterF;
use crate::xap::xap_trait::{IterOf, Xap};

pub struct F<X: Xap, G: Fn(&X::O) -> bool> {
    x: X,
    g: G,
}

impl<X: Xap, G: Fn(&X::O) -> bool> F<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap, G: Fn(&X::O) -> bool> Xap for F<X, G> {
    type I = X::I;

    type O = X::O;

    type Values<'i>
        = IterF<IterOf<'i, X>, &'i G>
    where
        Self: 'i;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values<'_> {
        IterF::new(self.x.xap(i).into_iter(), &self.g)
    }

    // transformations

    type Map<Q, H>
        = M<Self, Q, H>
    where
        H: Fn(Self::O) -> Q;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q,
    {
        todo!()
    }

    type Inspect<H>
        = Faker<Self::I, Self::O>
    where
        H: Fn(&Self::O);

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O),
    {
        todo!()
    }

    type Filter<H>
        = F<Self, H>
    where
        H: Fn(&Self::O) -> bool;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool,
    {
        todo!()
    }

    type FilterMap<Q, H>
        = Faker<Self::I, Q>
    where
        H: Fn(Self::O) -> Option<Q>;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q>,
    {
        todo!()
    }

    type FlatMap<V, H>
        = Faker<Self::I, V::Item>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V,
    {
        todo!()
    }
}
