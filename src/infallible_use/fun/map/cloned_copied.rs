use crate::infallible_use::fun::map::fn_trait::UMap;
use core::marker::PhantomData;

// cloned

pub struct UFnCloned<'a, U, I: Clone>(PhantomData<&'a (I, U)>);

impl<'a, U, I: Clone> Clone for UFnCloned<'a, U, I> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<'a, U, I: Clone> Copy for UFnCloned<'a, U, I> {}

unsafe impl<'a, U, I: Clone> Send for UFnCloned<'a, U, I> {}

impl<'a, U, I: Clone> UFnCloned<'a, U, I> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<'a, U, I: Clone> UMap for UFnCloned<'a, U, I> {
    type I = &'a I;

    type O = I;

    type U = U;

    #[inline(always)]
    fn map(&self, _: &mut Self::U, i: Self::I) -> Self::O {
        i.clone()
    }
}

// copied

pub struct UFnCopied<'a, U, I: Copy>(PhantomData<&'a (I, U)>);

impl<'a, U, I: Copy> Clone for UFnCopied<'a, U, I> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<'a, U, I: Copy> Copy for UFnCopied<'a, U, I> {}

unsafe impl<'a, U, I: Copy> Send for UFnCopied<'a, U, I> {}

impl<'a, U, I: Copy> UFnCopied<'a, U, I> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<'a, U, I: Copy> UMap for UFnCopied<'a, U, I> {
    type I = &'a I;

    type O = I;

    type U = U;

    #[inline(always)]
    fn map(&self, _: &mut Self::U, i: Self::I) -> Self::O {
        *i
    }
}
