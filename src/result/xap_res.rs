use crate::infallible::Xap;

pub trait XapRes {
    /// Type of the intermediate success value bridging between `X1` and `X2`.
    type M;

    /// Error type.
    type E;

    /// First part of the computation which exits infallible and enters fallible.
    type X1: Xap<O = Result<Self::M, Self::E>>;

    /// Second part of the computation that operates on the success type `M`.
    type X2: Xap<I = Self::M>;

    type SuccessValues;

    fn xap_res(&self, i: <Self::X1 as Xap>::I) -> Result<Self::SuccessValues, Self::E>;
}
