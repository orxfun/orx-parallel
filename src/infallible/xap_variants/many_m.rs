use crate::infallible::fun::filter_map::{FnFil, FnFilMap};
use crate::infallible::fun::flat_map::FnFlatMap;
use crate::infallible::fun::map::{FnIns, FnMap, Map};
use crate::infallible::size::Many;
use crate::infallible::xap::Xap;
use crate::infallible::xap_variants::many_f::ManyF;
use crate::infallible::xap_variants::many_x::ManyX;

pub struct ManyM<X: Xap<Size = Many>, G: Map<I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap<Size = Many>, G: Map<I = X::O>> Clone for ManyM<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: Xap<Size = Many>, G: Map<I = X::O>> Copy for ManyM<X, G> {}

impl<X: Xap<Size = Many>, G: Map<I = X::O>> ManyM<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap<Size = Many>, G: Map<I = X::O>> Xap for ManyM<X, G> {
    type I = X::I;

    type O = G::O;

    type Size = Many;

    type Values = IterManyM<<X::Values as IntoIterator>::IntoIter, G>;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values {
        let i = self.x.xap(i).into_iter();
        IterManyM { i, g: self.g }
    }

    // transformations

    type Map<Q, H>
        = ManyM<Self, FnMap<Self::O, Q, H>>
    where
        H: Fn(Self::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q + Copy + Send,
    {
        ManyM::new(self, FnMap::new(h))
    }

    type Inspect<H>
        = ManyM<Self, FnIns<Self::O, H>>
    where
        H: Fn(&Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O) + Copy + Send,
    {
        ManyM::new(self, FnIns::new(h))
    }

    type Filter<H>
        = ManyF<Self, FnFil<Self::O, H>>
    where
        H: Fn(&Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool + Copy + Send,
    {
        ManyF::new(self, FnFil::new(h))
    }

    type FilterMap<Q, H>
        = ManyF<Self, FnFilMap<Self::O, Q, H>>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send,
    {
        ManyF::new(self, FnFilMap::new(h))
    }

    type FlatMap<V, H>
        = ManyX<Self, FnFlatMap<Self::O, V, H>>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send,
    {
        ManyX::new(self, FnFlatMap::new(h))
    }

    // transformations - helper

    type Mapped<M>
        = ManyM<Self, M>
    where
        M: Map<I = Self::O>;

    fn mapped<M>(self, m: M) -> Self::Mapped<M>
    where
        M: Map<I = Self::O>,
    {
        ManyM::new(self, m)
    }
}

// iter

pub struct IterManyM<I, G>
where
    I: Iterator,
    G: Map<I = I::Item>,
{
    i: I,
    g: G,
}

impl<I, G> Iterator for IterManyM<I, G>
where
    I: Iterator,
    G: Map<I = I::Item>,
{
    type Item = G::O;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.i.next().map(|x| self.g.map(x))
    }
}
