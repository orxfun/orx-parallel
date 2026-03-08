pub trait FlatMapFn {
    type I;

    type O: IntoIterator;

    fn filter_map(&self, i: Self::I) -> Self::O;
}

impl<'a, X: FlatMapFn> FlatMapFn for &'a X {
    type I = X::I;

    type O = X::O;

    #[inline(always)]
    fn filter_map(&self, i: Self::I) -> Self::O {
        <X as FlatMapFn>::filter_map(self, i)
    }
}
