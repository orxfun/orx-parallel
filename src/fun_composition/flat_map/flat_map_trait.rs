pub trait FlatMap {
    type I;

    type O;

    type Vo: IntoIterator<Item = Self::O>;

    fn flat_map(&self, i: Self::I) -> Self::Vo;
}
