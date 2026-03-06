use crate::xap::xap_trait::Xap;
use core::marker::PhantomData;

pub struct Faker<I, O> {
    p: PhantomData<(I, O)>,
}

impl<I, O> Xap for Faker<I, O> {
    type I = I;

    type O = O;

    type Values<'i>
        = [O; 1]
    where
        Self: 'i;

    fn xap(&self, _: Self::I) -> Self::Values<'_> {
        todo!()
    }

    type Map<Q, G>
        = Faker<I, Q>
    where
        G: Fn(Self::O) -> Q;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q,
    {
        todo!()
    }

    type Inspect<G>
        = Faker<I, O>
    where
        G: Fn(&Self::O);

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O),
    {
        todo!()
    }

    type Filter<G>
        = Faker<I, O>
    where
        G: Fn(&Self::O) -> bool;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool,
    {
        todo!()
    }

    type FilterMap<Q, G>
        = Faker<I, Q>
    where
        G: Fn(Self::O) -> Option<Q>;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q>,
    {
        todo!()
    }

    type FlatMap<V, G>
        = Faker<I, V::Item>
    where
        V: IntoIterator,
        G: Fn(Self::O) -> V;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V,
    {
        todo!()
    }
}
