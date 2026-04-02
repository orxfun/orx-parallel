use crate::infallible_using::fun::map::fn_trait::MapU;
use core::marker::PhantomData;

// cloned

pub struct FnClonedU<'a, U, I: Clone>(PhantomData<&'a (I, U)>);

impl<'a, U, I: Clone> Clone for FnClonedU<'a, U, I> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<'a, U, I: Clone> Copy for FnClonedU<'a, U, I> {}

unsafe impl<'a, U, I: Clone> Send for FnClonedU<'a, U, I> {}

impl<'a, U, I: Clone> FnClonedU<'a, U, I> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<'a, U, I: Clone> MapU for FnClonedU<'a, U, I> {
    type I = &'a I;

    type O = I;

    type U = U;

    #[inline(always)]
    fn map(&self, _: &mut Self::U, i: Self::I) -> Self::O {
        i.clone()
    }
}

// copied

pub struct FnCopiedU<'a, U, I: Copy>(PhantomData<&'a (I, U)>);

impl<'a, U, I: Copy> Clone for FnCopiedU<'a, U, I> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<'a, U, I: Copy> Copy for FnCopiedU<'a, U, I> {}

unsafe impl<'a, U, I: Copy> Send for FnCopiedU<'a, U, I> {}

impl<'a, U, I: Copy> FnCopiedU<'a, U, I> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<'a, U, I: Copy> MapU for FnCopiedU<'a, U, I> {
    type I = &'a I;

    type O = I;

    type U = U;

    #[inline(always)]
    fn map(&self, _: &mut Self::U, i: Self::I) -> Self::O {
        *i
    }
}
