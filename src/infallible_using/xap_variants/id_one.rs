use crate::infallible::size::One;
use crate::infallible_using::fun::{FnFil, FnFilMap, FnFlatMap, FnIns, FnMap, Map};
use crate::infallible_using::xap::Xap;
use crate::infallible_using::xap_variants::{OneF, OneM, OneX};
use core::marker::PhantomData;

pub struct OneId<U, X: crate::infallible::Xap<Size = One>>(X, PhantomData<U>);

impl<U, X: crate::infallible::Xap<Size = One>> Clone for OneId<U, X> {
    fn clone(&self) -> Self {
        Self::new(self.0)
    }
}

impl<U, X: crate::infallible::Xap<Size = One>> Copy for OneId<U, X> {}

unsafe impl<U, X: crate::infallible::Xap<Size = One>> Send for OneId<U, X> {}

impl<U, X: crate::infallible::Xap<Size = One>> OneId<U, X> {
    pub const fn new(xap: X) -> Self {
        Self(xap, PhantomData)
    }
}

impl<U, X: crate::infallible::Xap<Size = One>> Xap for OneId<U, X> {
    type I = X::I;

    type O = X::O;

    type U = U;

    type Size = X::Size;

    type Values = X::Values;

    #[inline(always)]
    fn xap(&self, _: &mut Self::U, i: Self::I) -> Self::Values {
        self.0.xap(i)
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
