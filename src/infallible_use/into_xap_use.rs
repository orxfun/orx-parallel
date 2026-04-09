use crate::infallible::Xap;
use crate::infallible_use::sizes::SizeInfUse;
use crate::infallible_use::xap_variants::Id;
use crate::infallible_use::{XapUse, use_var::Use};
use crate::sizes::One;

pub trait IntoXapUse: SizeInfUse {
    type XapUse<X: Xap<Size = Self>, U: Use>: XapUse<U = U, Size = Self>;

    fn into_xap_use<X: Xap<Size = Self>, U: Use>(xap: X) -> Self::XapUse<X, U>;
}

impl IntoXapUse for One {
    type XapUse<X: Xap<Size = Self>, U: Use> = Id<X, U>;

    fn into_xap_use<X: Xap<Size = Self>, U: Use>(xap: X) -> Self::XapUse<X, U> {
        Id::new(xap)
    }
}
