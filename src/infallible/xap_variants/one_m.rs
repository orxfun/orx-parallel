use crate::infallible::fun::map::FnMap;
use crate::infallible::xap::{Xap, XapOne};
use crate::infallible::{fun::map::Map, size::One};

pub struct OneM<X: Xap<Size = One>, G: Map<I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap<Size = One>, G: Map<I = X::O>> Clone for OneM<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: Xap<Size = One>, G: Map<I = X::O>> Copy for OneM<X, G> {}

impl<X: Xap<Size = One>, G: Map<I = X::O>> OneM<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap<Size = One>, G: Map<I = X::O>> Xap for OneM<X, G> {
    type I = X::I;

    type O = G::O;

    type Size = One;

    type Values = [G::O; 1];

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values {
        [self.g.map(self.x.one_value(i))]
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

    type Inspect<H>
        = crate::infallible::xap::Fake<Self::I, Self::O>
    where
        H: Fn(&Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O) + Copy + Send,
    {
        todo!()
    }

    type Filter<H>
        = crate::infallible::xap::Fake<Self::I, Self::O>
    where
        H: Fn(&Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool + Copy + Send,
    {
        todo!()
    }

    type FilterMap<Q, H>
        = crate::infallible::xap::Fake<Self::I, Q>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send,
    {
        todo!()
    }

    type FlatMap<V, H>
        = crate::infallible::xap::Fake<Self::I, <V as IntoIterator>::Item>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send,
    {
        todo!()
    }
}
