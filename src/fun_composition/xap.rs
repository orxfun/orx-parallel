use crate::generic_values::{TransformableValues, Values};

type Elem<X> = <<X as Xap>::O as Values>::Item;

pub trait Xap {
    type I;

    type O: TransformableValues;

    fn xap(&self, i: Self::I) -> Self::O;

    // transformations

    type Map<X, Q>: Xap<I = Self::I, O = <Self::O as TransformableValues>::Map<X, Q>>
    where
        X: Fn(Elem<Self>) -> Q;
    fn map<X, Q>(self, map: X) -> Self::Map<X, Q>
    where
        X: Fn(Elem<Self>) -> Q;

    type Inspect<X>: Xap<I = Self::I, O = <Self::O as TransformableValues>::Inspect<X>>
    where
        X: Fn(&Elem<Self>);
    fn inspect<X>(self, inspect: X) -> Self::Inspect<X>
    where
        X: Fn(&Elem<Self>);

    type Filter<X>: Xap<I = Self::I, O = <Self::O as TransformableValues>::Filter<X>>
    where
        X: Fn(&Elem<Self>) -> bool;
    fn filter<X>(self, filter: X) -> Self::Filter<X>
    where
        X: Fn(&Elem<Self>) -> bool;

    type FlatMap<X, Q>: Xap<I = Self::I, O = <Self::O as TransformableValues>::FlatMap<X, Q>>
    where
        Q: IntoIterator,
        X: Fn(Elem<Self>) -> Q;
    fn flat_map<X, Q>(self, flat_map: X) -> Self::FlatMap<X, Q>
    where
        Q: IntoIterator,
        X: Fn(Elem<Self>) -> Q;

    type FilterMap<X, Q>: Xap<I = Self::I, O = <Self::O as TransformableValues>::FilterMap<X, Q>>
    where
        X: Fn(Elem<Self>) -> Option<Q>;
    fn filter_map<Fm, O>(self, filter_map: Fm) -> Self::FilterMap<Fm, O>
    where
        Fm: Fn(Elem<Self>) -> Option<O>;

    type Whilst<X>: Xap<I = Self::I, O = <Self::O as TransformableValues>::Whilst<X>>
    where
        X: Fn(&Elem<Self>) -> bool;
    fn whilst<W>(self, whilst: W) -> Self::Whilst<W>
    where
        W: Fn(&Elem<Self>) -> bool;
}
