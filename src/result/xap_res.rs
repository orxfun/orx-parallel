use crate::infallible::{fun::Map, xap::Xap};

pub type ResOf<X> = Result<<<X as XapRes>::X2 as Xap>::O, <X as XapRes>::E>;

pub trait XapRes {
    /// Type of the intermediate success value bridging between `X1` and `X2`.
    type M;

    /// Error type.
    type E;

    /// First part of the computation which exits infallible and enters fallible.
    type X1: Xap<O = Result<Self::M, Self::E>>;

    /// Second part of the computation that operates on the success type `M`.
    type X2: Xap<I = Self::M>;

    type Results: IntoIterator<Item = ResOf<Self>>;

    fn xap_res(&self, i: <Self::X1 as Xap>::I) -> Self::Results;

    // transformations

    type Map<Q, H>: XapRes<M = Self::M, E = Self::E, X1 = Self::X1, X2 = <Self::X2 as Xap>::Map<Q, H>>
    where
        H: Fn(<Self::X2 as Xap>::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(<Self::X2 as Xap>::O) -> Q + Copy + Send;

    type Inspect<H>: XapRes<M = Self::M, E = Self::E, X1 = Self::X1, X2 = <Self::X2 as Xap>::Inspect<H>>
    where
        H: Fn(&<Self::X2 as Xap>::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&<Self::X2 as Xap>::O) + Copy + Send;

    type Filter<H>: XapRes<M = Self::M, E = Self::E, X1 = Self::X1, X2 = <Self::X2 as Xap>::Filter<H>>
    where
        H: Fn(&<Self::X2 as Xap>::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&<Self::X2 as Xap>::O) -> bool + Copy + Send;

    type FilterMap<Q, H>: XapRes<M = Self::M, E = Self::E, X1 = Self::X1, X2 = <Self::X2 as Xap>::FilterMap<Q, H>>
    where
        H: Fn(<Self::X2 as Xap>::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(<Self::X2 as Xap>::O) -> Option<Q> + Copy + Send;

    type FlatMap<V, H>: XapRes<M = Self::M, E = Self::E, X1 = Self::X1, X2 = <Self::X2 as Xap>::FlatMap<V, H>>
    where
        V: IntoIterator,
        H: Fn(<Self::X2 as Xap>::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(<Self::X2 as Xap>::O) -> V + Copy + Send;

    // transformations - helper

    type Mapped<H>: XapRes<M = Self::M, E = Self::E, X1 = Self::X1, X2 = <Self::X2 as Xap>::Mapped<H>>
    where
        H: Map<I = <Self::X2 as Xap>::O>;

    fn mapped<M>(self, h: M) -> Self::Mapped<M>
    where
        M: Map<I = <Self::X2 as Xap>::O>;
}
