pub trait FilterMap: Copy + Send {
    type I;

    type O;

    fn filter_map(&self, i: Self::I) -> Option<Self::O>;
}
