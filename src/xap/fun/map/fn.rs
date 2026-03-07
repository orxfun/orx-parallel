pub trait MapFn {
    type I;

    type O;

    fn map(&self, i: Self::I) -> Self::O;
}
