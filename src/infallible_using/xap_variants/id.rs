use crate::infallible::size::One;
use crate::infallible_using::fun::{FnFilMapU, FnFilU, FnFlatMapU, FnInsU, FnMapU, MapU};
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

    type Values<'a>
        = [I; 1]
    where
        Self: 'a;

    fn xap<'a>(&self, _: &'a mut Self::U, i: Self::I) -> Self::Values<'a>
    where
        Self: 'a,
    {
        [i]
    }

    // transformations

    type Map<Q, H>
        = OneM<Self, FnMapU<Self::U, Self::O, Q, H>>
    where
        H: Fn(&mut Self::U, Self::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(&mut Self::U, Self::O) -> Q + Copy + Send,
    {
        OneM::new(self, FnMapU::new(h))
    }

    type Inspect<H>
        = OneM<Self, FnInsU<Self::U, Self::O, H>>
    where
        H: Fn(&mut Self::U, &Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&mut Self::U, &Self::O) + Copy + Send,
    {
        OneM::new(self, FnInsU::new(h))
    }

    type Filter<H>
        = OneF<Self, FnFilU<Self::U, Self::O, H>>
    where
        H: Fn(&mut Self::U, &Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&mut Self::U, &Self::O) -> bool + Copy + Send,
    {
        OneF::new(self, FnFilU::new(h))
    }

    type FilterMap<Q, H>
        = OneF<Self, FnFilMapU<Self::U, Self::O, Q, H>>
    where
        H: Fn(&mut Self::U, Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(&mut Self::U, Self::O) -> Option<Q> + Copy + Send,
    {
        OneF::new(self, FnFilMapU::new(h))
    }

    type FlatMap<V, H>
        = OneX<Self, FnFlatMapU<Self::U, Self::O, V, H>>
    where
        V: IntoIterator,
        H: Fn(&mut Self::U, Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(&mut Self::U, Self::O) -> V + Copy + Send,
    {
        OneX::new(self, FnFlatMapU::new(h))
    }

    // transformations - helper

    type Mapped<M>
        = OneM<Self, M>
    where
        M: MapU<U = Self::U, I = Self::O>;

    fn mapped<M>(self, m: M) -> Self::Mapped<M>
    where
        M: MapU<U = Self::U, I = Self::O>,
    {
        OneM::new(self, m)
    }
}
