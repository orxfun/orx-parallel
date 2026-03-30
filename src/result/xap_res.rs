use crate::infallible::Xap;

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

    // type Map<Q, H>: XapRes<M = Self::M, E = Self::E, X1 = Self::X1, X2 = <Self::X2 as Xap>::Map<Q, H>>
    // where
    //     H: Fn(<Self::X2 as Xap>::O) -> Q + Copy + Send;
}
