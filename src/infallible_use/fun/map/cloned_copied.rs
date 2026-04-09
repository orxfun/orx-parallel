use crate::infallible_use::fun::map::fn_trait::Map;
use core::marker::PhantomData;

// cloned

pub struct FnCloned<'a, U, I: Clone>(PhantomData<&'a (I, U)>);

impl<'a, U, I: Clone> Clone for FnCloned<'a, U, I> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<'a, U, I: Clone> Copy for FnCloned<'a, U, I> {}

unsafe impl<'a, U, I: Clone> Send for FnCloned<'a, U, I> {}

impl<'a, U, I: Clone> FnCloned<'a, U, I> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<'a, U, I: Clone> Map for FnCloned<'a, U, I> {
    type I = &'a I;

    type O = I;

    type U = U;

    #[inline(always)]
    fn map(&self, _: &mut Self::U, i: Self::I) -> Self::O {
        i.clone()
    }
}

// copied

pub struct FnCopied<'a, U, I: Copy>(PhantomData<&'a (I, U)>);

impl<'a, U, I: Copy> Clone for FnCopied<'a, U, I> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<'a, U, I: Copy> Copy for FnCopied<'a, U, I> {}

unsafe impl<'a, U, I: Copy> Send for FnCopied<'a, U, I> {}

impl<'a, U, I: Copy> FnCopied<'a, U, I> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<'a, U, I: Copy> Map for FnCopied<'a, U, I> {
    type I = &'a I;

    type O = I;

    type U = U;

    #[inline(always)]
    fn map(&self, _: &mut Self::U, i: Self::I) -> Self::O {
        *i
    }
}
