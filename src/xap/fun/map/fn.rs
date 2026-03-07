pub trait MapFn {
    type I;

    type O;

    fn map(&self, i: Self::I) -> Self::O;
}

impl<'a, X: MapFn> MapFn for &'a X {
    type I = X::I;

    type O = X::O;

    #[inline(always)]
    fn map(&self, i: Self::I) -> Self::O {
        <X as MapFn>::map(self, i)
    }
}
