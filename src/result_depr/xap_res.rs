use crate::infallible::{Xap, sizes::Size};

pub type InOf<X> = <<X as XapRes>::X1 as Xap>::I;

pub type OutOf<X> = <<X as XapRes>::X2 as Xap>::O;

pub type ResOf<X> = Result<OutOf<X>, <X as XapRes>::E>;

pub trait XapRes: Copy + Send {
    /// Type of the intermediate success value bridging between `X1` and `X2`.
    type M;

    /// Error type.
    type E;

    type X1: Xap<O = Result<Self::M, Self::E>>;

    type X2: Xap<I = Self::M>;

    type Size: Size;

    type Results: IntoIterator<Item = ResOf<Self>>;

    fn xap_res(&self, i: InOf<Self>) -> Self::Results;

    // // transformations

    // type Map<Q, H>: XapRes<I = Self::I, M = Self::M, E = Self::E, O = Q>
    // where
    //     H: Fn(Self::O) -> Q + Copy + Send;

    // fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    // where
    //     H: Fn(Self::O) -> Q + Copy + Send;

    // type Inspect<H>: XapRes<I = Self::I, M = Self::M, E = Self::E, O = Self::O>
    // where
    //     H: Fn(&Self::O) + Copy + Send;

    // fn inspect<H>(self, h: H) -> Self::Inspect<H>
    // where
    //     H: Fn(&Self::O) + Copy + Send;

    // type Filter<H>: XapRes<I = Self::I, M = Self::M, E = Self::E, O = Self::O>
    // where
    //     H: Fn(&Self::O) -> bool + Copy + Send;

    // fn filter<H>(self, h: H) -> Self::Filter<H>
    // where
    //     H: Fn(&Self::O) -> bool + Copy + Send;

    // type FilterMap<Q, H>: XapRes<I = Self::I, M = Self::M, E = Self::E, O = Q>
    // where
    //     H: Fn(Self::O) -> Option<Q> + Copy + Send;

    // fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    // where
    //     H: Fn(Self::O) -> Option<Q> + Copy + Send;

    // type FlatMap<V, H>: XapRes<I = Self::I, M = Self::M, E = Self::E, O = <V as IntoIterator>::Item>
    // where
    //     V: IntoIterator,
    //     H: Fn(Self::O) -> V + Copy + Send;

    // fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    // where
    //     V: IntoIterator,
    //     H: Fn(Self::O) -> V + Copy + Send;

    // // transformations - helper

    // type Mapped<H>: XapRes<I = Self::I, M = Self::M, E = Self::E, O = H::O>
    // where
    //     H: Map<I = Self::O>;

    // fn mapped<M>(self, h: M) -> Self::Mapped<M>
    // where
    //     M: Map<I = Self::O>;
}
