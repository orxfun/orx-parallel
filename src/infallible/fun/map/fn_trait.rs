pub trait Map: Copy + Send {
    type I;

    type O;

    fn map(&self, i: Self::I) -> Self::O;
}
