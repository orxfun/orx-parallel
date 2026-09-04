use crate::infallible::Xap;

/// Enumerable extension of [Xap].
pub trait XapEnumByInput: Xap {
    /// Type of the enumerated computation, which also implements [Xap].
    type Enumerated: Xap<I = (usize, Self::I), O = (usize, Self::O), Size = Self::Size>;

    /// Transforms the computation to enumerated one.
    fn enumerate(self) -> Self::Enumerated;
}
