pub trait FilterFn {
    type I;

    fn filter(&self, i: &Self::I) -> bool;
}

impl<'a, X: FilterFn> FilterFn for X {
    type I = X::I;

    #[inline(always)]
    fn filter(&self, i: &Self::I) -> bool {
        <Self as FilterFn>::filter(self, i)
    }
}
