use crate::infallible_using::XapUse;

pub trait XapUseEnumByInput: XapUse {
    type Enumerated: XapUse<U = Self::U, I = (usize, Self::I), O = (usize, Self::O), Size = Self::Size>;

    fn enumerate(self) -> Self::Enumerated;
}
