use crate::infallible::Xap;
use crate::infallible_use::fun::{Map, MapEnum};
use crate::infallible_use::{XapOne, XapUse, XapUseEnumByInput};
use crate::sizes::One;
use core::marker::PhantomData;

pub struct IdOne<X: Xap<Size = One>, U> {
    x: X,
    p: PhantomData<U>,
}

impl<X: Xap<Size = One>, U> IdOne<X, U> {
    pub fn new(x: X) -> Self {
        let p = PhantomData;
        Self { x, p }
    }
}

// impl<X: Xap<Size = One>, U> XapUse for IdOne<X, U> {
//     type U = U;

//     type I = X::I;

//     type O = X::O;

//     type Size = One;

//     type Values = X::Values;

//     fn xap(&self, _: &mut Self::U, i: Self::I) -> Self::Values {
//         todo!()
//     }
// }
