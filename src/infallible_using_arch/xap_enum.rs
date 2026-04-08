use crate::infallible_using::xap::Xap;

pub trait XapEnumByInput: Xap {
    type Enumerated: Xap<U = Self::U, I = (usize, Self::I), O = (usize, Self::O), Size = Self::Size>;

    fn enumerate(self) -> Self::Enumerated;
}
