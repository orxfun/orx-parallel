use crate::xap::{count::One, xap_trait::Xap};
use core::marker::PhantomData;

pub struct Faker<I, O> {
    p: PhantomData<(I, O)>,
}

impl<I, O> Xap for Faker<I, O> {
    type I = I;

    type O = O;

    type Count = One;

    type Values<'i>
        = [O; 1]
    where
        Self: 'i;

    fn xap(&self, _: Self::I) -> Self::Values<'_> {
        todo!()
    }

    // transformations

    type Map<Q, H>
        = Faker<I, Q>
    where
        H: Fn(Self::O) -> Q;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q,
    {
        todo!()
    }

    type Inspect<H>
        = Faker<I, O>
    where
        H: Fn(&Self::O);

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O),
    {
        todo!()
    }

    type Filter<H>
        = Faker<I, O>
    where
        H: Fn(&Self::O) -> bool;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool,
    {
        todo!()
    }

    type FilterMap<Q, H>
        = Faker<I, Q>
    where
        H: Fn(Self::O) -> Option<Q>;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q>,
    {
        todo!()
    }

    type FlatMap<V, H>
        = Faker<I, V::Item>
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
