pub trait Map: Copy {
    type I;

    type O;

    fn map(&self, i: Self::I) -> Self::O;
}

impl<'a, X: Map> Map for &'a X {
    type I = X::I;

    type O = X::O;

    #[inline(always)]
    fn map(&self, i: Self::I) -> Self::O {
        <X as Map>::map(self, i)
    }
}
