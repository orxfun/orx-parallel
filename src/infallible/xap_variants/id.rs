use crate::infallible::XapEnumerable;
use crate::infallible::fun::FnFlatMap;
use crate::infallible::fun::{FnFil, FnFilMap};
use crate::infallible::fun::{FnIns, FnMap, Map};
use crate::infallible::size::One;
use crate::infallible::xap::Xap;
use crate::infallible::xap_variants::one_f::OneF;
use crate::infallible::xap_variants::one_m::OneM;
use crate::infallible::xap_variants::one_x::OneX;
use core::marker::PhantomData;

pub struct Id<I>(PhantomData<I>);

impl<I> Clone for Id<I> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<I> Copy for Id<I> {}

unsafe impl<I> Send for Id<I> {}

impl<I> Id<I> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<I> XapEnumerable for Id<I> {
    type Enumerated = Id<(usize, I)>;

    fn enumerate(self) -> Self::Enumerated {
        Id::new()
    }
}

impl<I> Xap for Id<I> {
    type I = I;

    type O = I;

    type Size = One;

    type Values = [I; 1];

    fn xap(&self, i: Self::I) -> Self::Values {
        [i]
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
        = OneM<Self, FnIns<Self::O, H>>
    where
        H: Fn(&Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O) + Copy + Send,
    {
        OneM::new(self, FnIns::new(h))
    }

    type Filter<H>
        = OneF<Self, FnFil<Self::O, H>>
    where
        H: Fn(&Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool + Copy + Send,
    {
        OneF::new(self, FnFil::new(h))
    }

    type FilterMap<Q, H>
        = OneF<Self, FnFilMap<Self::O, Q, H>>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send,
    {
        OneF::new(self, FnFilMap::new(h))
    }

    type FlatMap<V, H>
        = OneX<Self, FnFlatMap<Self::O, V, H>>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send,
    {
        OneX::new(self, FnFlatMap::new(h))
    }

    // transformations - helper

    type Mapped<M>
        = OneM<Self, M>
    where
        M: Map<I = Self::O>;

    fn mapped<M>(self, m: M) -> Self::Mapped<M>
    where
        M: Map<I = Self::O>,
    {
        OneM::new(self, m)
    }
}
