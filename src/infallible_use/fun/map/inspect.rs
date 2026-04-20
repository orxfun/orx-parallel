use crate::infallible_use::fun::map::fn_trait::UMap;
use core::marker::PhantomData;

pub struct UFnIns<U, I, F: Fn(&mut U, &I) + Copy + Send>(F, PhantomData<(I, U)>);

impl<U, I, F: Fn(&mut U, &I) + Copy + Send> Clone for UFnIns<U, I, F> {
    fn clone(&self) -> Self {
        Self::new(self.0)
    }
}

impl<U, I, F: Fn(&mut U, &I) + Copy + Send> Copy for UFnIns<U, I, F> {}

unsafe impl<U, I, F: Fn(&mut U, &I) + Copy + Send> Send for UFnIns<U, I, F> {}

impl<U, I, F: Fn(&mut U, &I) + Copy + Send> UFnIns<U, I, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<U, I, F: Fn(&mut U, &I) + Copy + Send> UMap for UFnIns<U, I, F> {
    type I = I;

    type O = I;

    type U = U;

    #[inline(always)]
    fn map(&self, u: &mut Self::U, i: Self::I) -> Self::O {
        (self.0)(u, &i);
        i
    }
}
