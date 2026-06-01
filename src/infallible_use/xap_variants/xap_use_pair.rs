use crate::{infallible_use::XapUse, use_var::PairPtr};
use core::marker::PhantomData;

pub struct XapUsePair<X: XapUse, V: Send> {
    x: X,
    p: PhantomData<V>,
}

impl<X: XapUse, V: Send> XapUsePair<X, V> {
    pub fn new(x: X) -> Self {
        let p = PhantomData;
        Self { x, p }
    }
}

impl<X: XapUse, V: Send> Clone for XapUsePair<X, V> {
    fn clone(&self) -> Self {
        Self {
            x: self.x,
            p: PhantomData,
        }
    }
}

impl<X: XapUse, V: Send> Copy for XapUsePair<X, V> {}

impl<X: XapUse, V: Send> XapUse for XapUsePair<X, V> {
    type I = X::I;

    type O = X::O;

    type Size = X::Size;

    type Values = X::Values;

    type U = PairPtr<X::U, V>;

    fn xap_use(&self, u: *mut Self::U, i: Self::I) -> Self::Values {
        let pair_ptr: &mut PairPtr<X::U, V> = unsafe { &mut *u };
        let u = pair_ptr.u_mut();
        self.x.xap_use(u, i)
    }
}
