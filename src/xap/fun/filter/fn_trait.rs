pub trait Filter {
    type I;

    fn filter(&self, i: &Self::I) -> bool;
}

impl<'a, X: Filter> Filter for &'a X {
    type I = X::I;

    #[inline(always)]
    fn filter(&self, i: &Self::I) -> bool {
        <X as Filter>::filter(self, i)
    }
}
