pub trait FilterMap {
    type I;

    type O;

    fn filter_map(&self, i: Self::I) -> Option<Self::O>;
}
