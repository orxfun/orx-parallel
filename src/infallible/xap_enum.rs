use crate::infallible::Xap;

pub trait XapEnumerable: Xap {
    type Enumerated;

    fn enumerate(self) -> Self::Enumerated;
}
