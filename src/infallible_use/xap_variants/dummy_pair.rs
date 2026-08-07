use crate::{infallible_use::XapUse, use_var::PairPtr};
use core::marker::PhantomData;

pub struct UDummyPair<X: XapUse, V: Send> {
    x: X,
    p: PhantomData<V>,
}

impl<X: XapUse, V: Send> UDummyPair<X, V> {
    pub fn new(x: X) -> Self {
        let p = PhantomData;
        Self { x, p }
    }
}

impl<X: XapUse, V: Send> Clone for UDummyPair<X, V> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<X: XapUse, V: Send> Copy for UDummyPair<X, V> {}

impl<X: XapUse, V: Send> XapUse for UDummyPair<X, V> {
    type I = X::I;

    type O = X::O;

    type Size = X::Size;

    type Values = X::Values;

    type U = PairPtr<X::U, V>;

    fn xap_use(&self, u: *mut Self::U, i: Self::I) -> Self::Values {
        let pair_ptr: &mut PairPtr<X::U, V> = unsafe { &mut *u };
        let u = pair_ptr.u_ptr();
        self.x.xap_use(u, i)
    }
}
