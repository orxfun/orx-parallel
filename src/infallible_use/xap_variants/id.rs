use crate::infallible::{Xap, XapEnumByInput};
use crate::infallible_use::sizes::SizeInfUse;
use crate::infallible_use::{XapUse, XapUseEnumByInput};
use crate::sizes::One;
use core::marker::PhantomData;

pub struct Id<X: Xap, U>
where
    X::Size: SizeInfUse,
{
    x: X,
    p: PhantomData<U>,
}

impl<X: Xap, U> Clone for Id<X, U>
where
    X::Size: SizeInfUse,
{
    fn clone(&self) -> Self {
        Self {
            x: self.x,
            p: PhantomData,
        }
    }
}

impl<X: Xap, U> Copy for Id<X, U> where X::Size: SizeInfUse {}

unsafe impl<X: Xap, U> Send for Id<X, U> where X::Size: SizeInfUse {}

impl<X: Xap, U> Id<X, U>
where
    X::Size: SizeInfUse,
{
    pub fn new(x: X) -> Self {
        let p = PhantomData;
        Self { x, p }
    }
}

impl<X: Xap, U> XapUse for Id<X, U>
where
    X::Size: SizeInfUse,
{
    type U = U;

    type I = X::I;

    type O = X::O;

    type Size = X::Size;

    type Values = X::Values;

    #[inline(always)]
    fn xap_use(&self, _: &mut Self::U, i: Self::I) -> Self::Values {
        self.x.xap(i)
    }
}

impl<X: XapEnumByInput<Size = One>, U> XapUseEnumByInput for Id<X, U>
where
    X::Size: SizeInfUse,
{
    type Enumerated = Id<X::Enumerated, U>;

    fn enumerate(self) -> Self::Enumerated {
        let x = self.x.enumerate();
        Id::new(x)
    }
}
