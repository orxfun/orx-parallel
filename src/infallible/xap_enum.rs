use crate::infallible::Xap;

pub trait XapEnumerable: Xap {
    type Enumerated: Xap<I = (usize, Self::I), O = (usize, Self::O), Size = Self::Size>;

    fn enumerate(self) -> Self::Enumerated;
}
