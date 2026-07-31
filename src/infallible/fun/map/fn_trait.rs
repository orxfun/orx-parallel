/// Function-like mapper from one value to one value.
pub trait Map: Copy + Send {
    /// Input type.
    type I;

    /// Output type.
    type O;

    /// Maps one input value.
    fn map(&self, i: Self::I) -> Self::O;
}
