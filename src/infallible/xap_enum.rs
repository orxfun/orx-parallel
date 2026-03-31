use crate::infallible::Xap;

pub trait XapEnumerate: Xap {
    type Enumerated: Xap<I = Self::I, O = (usize, Self::O), Size = Self::Size>;

    fn enumerate(self) -> Self::Enumerated;
}
