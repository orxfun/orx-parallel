use crate::xap::count::Count;
use crate::xap::faker::Faker;
use crate::xap::fun::filter::FilterQ;
use crate::xap::fun::map::{MapS, MapWrap};
use crate::xap::xap_implementors::fil_m::FilM;
use crate::xap::xap_implementors::fla_m::FlaM;
use crate::xap::xap_implementors::ins::Ins;
use crate::xap::xap_implementors::m::M;
use crate::xap::xap_implementors::xap_iters::IterF;
use crate::xap::xap_trait::{IterOf, Xap};

pub struct F<X: Xap, G: FilterQ<I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap, G: FilterQ<I = X::O>> F<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap, G: FilterQ<I = X::O>> Xap for F<X, G> {
    type I = X::I;

    type O = X::I;

    type Count = <X::Count as Count>::ThenZeroOne;

    type Values<'i>
        = [Self::O; 1]
    where
        Self: 'i;

    fn xap(&self, i: Self::I) -> Self::Values<'_> {
        todo!()
    }

    // transformations

    type Map<Q, H>
        = Faker<Self::I, Q>
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
        = Faker<Self::I, Self::O>
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
