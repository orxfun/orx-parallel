pub trait FilterMapFn {
    type I;

    type O;

    fn filter_map(&self, i: Self::I) -> Option<Self::O>;
}

impl<'a, X: FilterMapFn> FilterMapFn for &'a X {
    type I = X::I;

    type O = X::O;

    #[inline(always)]
    fn filter_map(&self, i: Self::I) -> Option<Self::O> {
        <X as FilterMapFn>::filter_map(self, i)
    }
}
