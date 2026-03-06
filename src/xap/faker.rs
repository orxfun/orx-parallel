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

    type Inspect<G>
        = Faker<I, O>
    where
        G: Fn(&Self::O);

    type Filter<G>
        = Faker<I, O>
    where
        G: Fn(&Self::O) -> bool;

    type FilterMap<Q, G>
        = Faker<I, Q>
    where
        G: Fn(Self::O) -> Option<Q>;

    type FlatMap<V, G>
        = Faker<I, V::Item>
    where
        V: IntoIterator,
        G: Fn(Self::O) -> V;
}
