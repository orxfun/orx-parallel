pub trait FilterMap: Copy {
    type I;

    type O;

    fn filter_map(&self, i: Self::I) -> Option<Self::O>;
}

impl<'a, X: FilterMap> FilterMap for &'a X {
    type I = X::I;

    type O = X::O;

    #[inline(always)]
    fn filter_map(&self, i: Self::I) -> Option<Self::O> {
        <X as FilterMap>::filter_map(self, i)
    }
}
