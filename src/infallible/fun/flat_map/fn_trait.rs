/// Function-like mapper from one value to many values.
pub trait FlatMap: Copy + Send {
    /// Input type.
    type I;

    /// Output iterator container type.
    type O: IntoIterator;

    /// Maps one input value to many outputs.
    fn flat_map(&self, i: Self::I) -> Self::O;
}
