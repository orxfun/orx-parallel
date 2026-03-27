use crate::xap::count::Count;

pub type Out<X> = Option<<X as XapOpt>::O>;

pub trait XapOpt: Copy + Send {
    type I;

    type O;

    type Count: Count;

    type Values: IntoIterator<Item = Out<Self>>;

    fn xap(&self, i: Self::I) -> Self::Values;
    fn into_iter_over(
        self,
        inputs: impl IntoIterator<Item = Self::I>,
    ) -> impl Iterator<Item = Out<Self>>;

    // transformations

    type Map<Q, H>: XapOpt<I = Self::I, O = Q>
    where
        H: Fn(Self::O) -> Q + Copy + Send;
}
