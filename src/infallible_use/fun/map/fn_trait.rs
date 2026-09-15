pub trait UMap: Copy + Send {
    type I;

    type O;

    type U;

    fn map(&self, u: &mut Self::U, i: Self::I) -> Self::O;
}
