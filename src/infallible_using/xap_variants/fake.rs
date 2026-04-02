use crate::{
    infallible::size::{Many, Size},
    infallible_using::{fun::MapU, xap::Xap},
};
use core::marker::PhantomData;

pub struct Fake<I, O, U, S: Size>(PhantomData<(I, O, U, S)>);

impl<I, O, U, S: Size> Clone for Fake<I, O, U, S> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<I, O, U, S: Size> Copy for Fake<I, O, U, S> {}

unsafe impl<I, O, U, S: Size> Send for Fake<I, O, U, S> {}

impl<I, O, U, S: Size> Xap for Fake<I, O, U, S> {
    type I = I;

    type O = O;

    type U = U;

    type Size = S;

    type Values<'a>
        = core::iter::Empty<O>
    where
        Self: 'a;

    fn xap<'a>(&self, u: &'a mut Self::U, i: Self::I) -> Self::Values<'_>
    where
        Self: 'a,
    {
        todo!()
    }

    // transformations

    type Map<Q, H>
        = Fake<Self::I, Q, Self::U, Self::Size>
    where
        H: Fn(&mut Self::U, Self::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(&mut Self::U, Self::O) -> Q + Copy + Send,
    {
        todo!()
    }

    type Inspect<H>
        = Fake<Self::I, Self::O, Self::U, Self::Size>
    where
        H: Fn(&mut Self::U, &Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&mut Self::U, &Self::O) + Copy + Send,
    {
        todo!()
    }

    type Filter<H>
        = Fake<Self::I, Self::O, Self::U, <Self::Size as Size>::ThenBin>
    where
        H: Fn(&mut Self::U, &Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&mut Self::U, &Self::O) -> bool + Copy + Send,
    {
        todo!()
    }

    type FilterMap<Q, H>
        = Fake<Self::I, Q, Self::U, <Self::Size as Size>::ThenBin>
    where
        H: Fn(&mut Self::U, Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(&mut Self::U, Self::O) -> Option<Q> + Copy + Send,
    {
        todo!()
    }

    type FlatMap<V, H>
        = Fake<Self::I, V::Item, Self::U, Many>
    where
        V: IntoIterator,
        H: Fn(&mut Self::U, Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(&mut Self::U, Self::O) -> V + Copy + Send,
    {
        todo!()
    }

    type Mapped<M>
        = Fake<Self::I, M::O, Self::U, Self::Size>
    where
        M: MapU<U = Self::U, I = Self::O>;

    fn mapped<M>(self, m: M) -> Self::Mapped<M>
    where
        M: MapU<U = Self::U, I = Self::O>,
    {
        todo!()
    }
}
