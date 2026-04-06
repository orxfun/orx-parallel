use crate::infallible::size::One;
use crate::infallible_using::fun::{FnFilMap, FnFil, FnFlatMap, FnIns, FnMap, Map};
use crate::infallible_using::xap::Xap;
use crate::infallible_using::xap_variants::{OneF, OneM, OneX};
use core::marker::PhantomData;

pub struct Id<U, I>(PhantomData<(U, I)>);

impl<U, I> Clone for Id<U, I> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<U, I> Copy for Id<U, I> {}

unsafe impl<U, I> Send for Id<U, I> {}

impl<U, I> Id<U, I> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<U, I> Xap for Id<U, I> {
    type I = I;

    type O = I;

    type U = U;

    type Size = One;

    type Values = [I; 1];

    fn xap(&self, _: &mut Self::U, i: Self::I) -> Self::Values {
        [i]
    }

    // transformations

    type Map<Q, H>
        = OneM<Self, FnMap<Self::U, Self::O, Q, H>>
    where
        H: Fn(&mut Self::U, Self::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(&mut Self::U, Self::O) -> Q + Copy + Send,
    {
        OneM::new(self, FnMap::new(h))
    }

    type Inspect<H>
        = OneM<Self, FnIns<Self::U, Self::O, H>>
    where
        H: Fn(&mut Self::U, &Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&mut Self::U, &Self::O) + Copy + Send,
    {
        OneM::new(self, FnIns::new(h))
    }

    type Filter<H>
        = OneF<Self, FnFil<Self::U, Self::O, H>>
    where
        H: Fn(&mut Self::U, &Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&mut Self::U, &Self::O) -> bool + Copy + Send,
    {
        OneF::new(self, FnFil::new(h))
    }

    type FilterMap<Q, H>
        = OneF<Self, FnFilMap<Self::U, Self::O, Q, H>>
    where
        H: Fn(&mut Self::U, Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(&mut Self::U, Self::O) -> Option<Q> + Copy + Send,
    {
        OneF::new(self, FnFilMap::new(h))
    }

    type FlatMap<V, H>
        = OneX<Self, FnFlatMap<Self::U, Self::O, V, H>>
    where
        V: IntoIterator,
        H: Fn(&mut Self::U, Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(&mut Self::U, Self::O) -> V + Copy + Send,
    {
        OneX::new(self, FnFlatMap::new(h))
    }

    // transformations - helper

    type Mapped<M>
        = OneM<Self, M>
    where
        M: Map<U = Self::U, I = Self::O>;

    fn mapped<M>(self, m: M) -> Self::Mapped<M>
    where
        M: Map<U = Self::U, I = Self::O>,
    {
        OneM::new(self, m)
    }
}
