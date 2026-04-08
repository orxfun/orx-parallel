pub trait FilterMap: Copy + Send {
    type I;

    type O;

    type U;

    fn filter_map(&self, u: &mut Self::U, i: Self::I) -> Option<Self::O>;
}
