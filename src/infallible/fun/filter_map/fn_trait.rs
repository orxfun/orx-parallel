/// Function-like mapper from one value to zero or one value.
pub trait FilterMap: Copy + Send {
    /// Input type.
    type I;

    /// Output type.
    type O;

    /// Maps one input value to an optional output.
    fn filter_map(&self, i: Self::I) -> Option<Self::O>;
}
