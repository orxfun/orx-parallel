pub trait MapU: Copy + Send {
    type I;

    type O;

    type U;

    fn map(&self, u: &mut Self::U, i: Self::I) -> Self::O;
}
