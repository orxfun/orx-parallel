use crate::infallible::Xap;

#[derive(Clone, Copy)]
pub struct XapSync<X: Xap>(X);

impl<X: Xap> XapSync<X> {
    pub(super) fn new(xap: X) -> Self {
        Self(xap)
    }
}

impl<X: Xap> Xap for XapSync<X> {
    type I = X::I;

    type O = X::O;

    type Size = X::Size;

    type Values = X::Values;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values {
        self.0.xap(i)
    }
}

unsafe impl<X: Xap> Sync for XapSync<X> {}
