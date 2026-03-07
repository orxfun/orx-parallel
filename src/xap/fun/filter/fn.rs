pub trait FilterFn {
    type I;

    fn filter(&self, i: &Self::I) -> bool;
}

impl<'a, X: FilterFn> FilterFn for &'a X {
    type I = X::I;

    #[inline(always)]
    fn filter(&self, i: &Self::I) -> bool {
        <X as FilterFn>::filter(self, i)
    }
}
