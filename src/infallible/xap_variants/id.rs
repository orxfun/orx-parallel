use crate::infallible::XapEnumByInput;
use crate::infallible::xap::Xap;
use crate::sizes::One;
use core::marker::PhantomData;

pub struct Id<I>(PhantomData<I>);

impl<I> Clone for Id<I> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<I> Copy for Id<I> {}

unsafe impl<I> Send for Id<I> {}

impl<I> Id<I> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<I> XapEnumByInput for Id<I> {
    type Enumerated = Id<(usize, I)>;

    fn enumerate(self) -> Self::Enumerated {
        Id::new()
    }
}

impl<I> Xap for Id<I> {
    type I = I;

    type O = I;

    type Size = One;

    type Values = [I; 1];

    fn xap(&self, i: Self::I) -> Self::Values {
        [i]
    }
}
