pub trait FlatMapU: Copy + Send {
    type I;

    type O: IntoIterator;

    type U;

    fn flat_map(&self, u: &mut Self::U, i: Self::I) -> Self::O;
}
