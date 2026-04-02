use crate::infallible_using::fun::map::fn_trait::MapU;
use core::marker::PhantomData;

pub struct FnInsU<U, I, F: Fn(&mut U, &I) + Copy + Send>(F, PhantomData<(I, U)>);

impl<U, I, F: Fn(&mut U, &I) + Copy + Send> Clone for FnInsU<U, I, F> {
    fn clone(&self) -> Self {
        Self::new(self.0)
    }
}

impl<U, I, F: Fn(&mut U, &I) + Copy + Send> Copy for FnInsU<U, I, F> {}

unsafe impl<U, I, F: Fn(&mut U, &I) + Copy + Send> Send for FnInsU<U, I, F> {}

impl<U, I, F: Fn(&mut U, &I) + Copy + Send> FnInsU<U, I, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<U, I, F: Fn(&mut U, &I) + Copy + Send> MapU for FnInsU<U, I, F> {
    type I = I;

    type O = I;

    type U = U;

    #[inline(always)]
    fn map(&self, u: &mut Self::U, i: Self::I) -> Self::O {
        (self.0)(u, &i);
        i
    }
}
