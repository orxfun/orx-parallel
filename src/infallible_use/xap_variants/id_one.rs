use crate::infallible::{Xap, XapEnumByInput};
use crate::infallible_use::{XapUse, XapUseEnumByInput};
use crate::sizes::One;
use core::marker::PhantomData;

pub struct IdOne<X: Xap<Size = One>, U> {
    x: X,
    p: PhantomData<U>,
}

impl<X: Xap<Size = One>, U> Clone for IdOne<X, U> {
    fn clone(&self) -> Self {
        Self {
            x: self.x,
            p: PhantomData,
        }
    }
}

impl<X: Xap<Size = One>, U> Copy for IdOne<X, U> {}

unsafe impl<X: Xap<Size = One>, U> Send for IdOne<X, U> {}

impl<X: Xap<Size = One>, U> IdOne<X, U> {
    pub fn new(x: X) -> Self {
        let p = PhantomData;
        Self { x, p }
    }
}

impl<X: Xap<Size = One>, U> XapUse for IdOne<X, U> {
    type U = U;

    type I = X::I;

    type O = X::O;

    type Size = One;

    type Values = X::Values;

    #[inline(always)]
    fn xap_use(&self, _: &mut Self::U, i: Self::I) -> Self::Values {
        self.x.xap(i)
    }
}

impl<X: XapEnumByInput<Size = One>, U> XapUseEnumByInput for IdOne<X, U> {
    type Enumerated = IdOne<X::Enumerated, U>;

    fn enumerate(self) -> Self::Enumerated {
        let x = self.x.enumerate();
        IdOne::new(x)
    }
}
