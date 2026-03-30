use crate::infallible::fun::filter_map::{FnFil, FnFilMap};
use crate::infallible::fun::flat_map::FnFlatMap;
use crate::infallible::fun::map::{FnIns, FnMap, Map};
use crate::infallible::size::One;
use crate::infallible::xap::Xap;
use crate::infallible::xap_variants::one_m::OneM;
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
