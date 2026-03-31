pub trait FlatMap {
    type I;

    type O;

    fn flat_map(&self, i: Self::I) -> impl IntoIterator<Item = Self::O>;
}
