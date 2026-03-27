pub trait FlatMap: Copy + Send {
    type I;

    type O: IntoIterator;

    fn flat_map(&self, i: Self::I) -> Self::O;
}
