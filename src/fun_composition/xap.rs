use crate::generic_values::{TransformableValues, Values};

pub type Elem<X> = <<X as Xap>::O as Values>::Item;

pub trait Xap {
    type I;

    type O: TransformableValues;

    fn xap(&self, i: Self::I) -> Self::O;

    // transformations

    type Map<G, Q>: Xap<I = Self::I, O = <Self::O as TransformableValues>::Map<G, Q>>
    where
        G: Fn(Elem<Self>) -> Q;
    fn map<G, Q>(self, map: G) -> Self::Map<G, Q>
    where
        G: Fn(Elem<Self>) -> Q;

    type Inspect<G>: Xap<I = Self::I, O = <Self::O as TransformableValues>::Inspect<G>>
    where
        G: Fn(&Elem<Self>);
    fn inspect<G>(self, inspect: G) -> Self::Inspect<G>
    where
        G: Fn(&Elem<Self>);

    type Filter<G>: Xap<I = Self::I, O = <Self::O as TransformableValues>::Filter<G>>
    where
        G: Fn(&Elem<Self>) -> bool;
    fn filter<X>(self, filter: X) -> Self::Filter<X>
    where
        X: Fn(&Elem<Self>) -> bool;

    type FlatMap<G, Q>: Xap<I = Self::I, O = <Self::O as TransformableValues>::FlatMap<G, Q>>
    where
        Q: IntoIterator,
        G: Fn(Elem<Self>) -> Q;
    fn flat_map<G, Q>(self, flat_map: G) -> Self::FlatMap<G, Q>
    where
        Q: IntoIterator,
        G: Fn(Elem<Self>) -> Q;

    type FilterMap<G, Q>: Xap<I = Self::I, O = <Self::O as TransformableValues>::FilterMap<G, Q>>
    where
        G: Fn(Elem<Self>) -> Option<Q>;
    fn filter_map<G, Q>(self, filter_map: G) -> Self::FilterMap<G, Q>
    where
        G: Fn(Elem<Self>) -> Option<Q>;

    type Whilst<G>: Xap<I = Self::I, O = <Self::O as TransformableValues>::Whilst<G>>
    where
        G: Fn(&Elem<Self>) -> bool;
    fn whilst<G>(self, whilst: G) -> Self::Whilst<G>
    where
        G: Fn(&Elem<Self>) -> bool;
}
