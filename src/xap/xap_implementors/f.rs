use crate::xap::M;
use crate::xap::count::Count;
use crate::xap::faker::Faker;
use crate::xap::fun::filter::{FWr, FilterQueue};
use crate::xap::fun::map::{MWr, Ms};
use crate::xap::xap_trait::Xap;

pub struct F<X: Xap, G: FilterQueue<I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap, G: FilterQueue<I = X::O>> F<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap, G: FilterQueue<I = X::O>> Xap for F<X, G> {
    type I = X::I;

    type O = X::O;

    type Count = <X::Count as Count>::ThenZeroOne;

    type Values<'i>
        = <Self::Count as Count>::Filter<X::Values<'i>, &'i G>
    where
        Self: 'i;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values<'_> {
        <Self::Count as Count>::filter(self.x.xap(i), &self.g)
    }

    // transformations

    type Map<Q, H>
        = M<Self, Ms<MWr<Self::O, Q, H>>>
    where
        H: Fn(Self::O) -> Q;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q,
    {
        M::new(self, Ms::new(MWr::new(h)))
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
        = F<X, G::Then<FWr<G::I, H>>>
    where
        H: Fn(&Self::O) -> bool;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool,
    {
        F::new(self.x, self.g.then(FWr::new(h)))
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
