use crate::infallible::fun::Map;
use core::marker::PhantomData;

// cloned

pub struct FnCloned<'a, I: Clone>(PhantomData<&'a I>);

impl<'a, I: Clone> Clone for FnCloned<'a, I> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<'a, I: Clone> Copy for FnCloned<'a, I> {}

unsafe impl<'a, I: Clone> Send for FnCloned<'a, I> {}

impl<'a, I: Clone> FnCloned<'a, I> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<'a, I: Clone> Map for FnCloned<'a, I> {
    type I = &'a I;

    type O = I;

    #[inline(always)]
    fn map(&self, i: Self::I) -> Self::O {
        i.clone()
    }
}

// copied

pub struct FnCopied<'a, I: Copy>(PhantomData<&'a I>);

impl<'a, I: Copy> Clone for FnCopied<'a, I> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<'a, I: Copy> Copy for FnCopied<'a, I> {}

unsafe impl<'a, I: Copy> Send for FnCopied<'a, I> {}

impl<'a, I: Copy> FnCopied<'a, I> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<'a, I: Copy> Map for FnCopied<'a, I> {
    type I = &'a I;

    type O = I;

    #[inline(always)]
    fn map(&self, i: Self::I) -> Self::O {
        *i
    }
}
