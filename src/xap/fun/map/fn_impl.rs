use crate::xap::fun::map::fn_trait::Map;
use core::marker::PhantomData;

// map

pub struct FnMap<I, O, F: Fn(I) -> O + Copy>(F, PhantomData<(I, O)>);

impl<I, O, F: Fn(I) -> O + Copy> Clone for FnMap<I, O, F> {
    fn clone(&self) -> Self {
        Self::new(self.0)
    }
}

impl<I, O, F: Fn(I) -> O + Copy> Copy for FnMap<I, O, F> {}

impl<I, O, F: Fn(I) -> O + Copy> FnMap<I, O, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<I, O, F: Fn(I) -> O + Copy> Map for FnMap<I, O, F> {
    type I = I;

    type O = O;

    #[inline(always)]
    fn map(&self, i: Self::I) -> Self::O {
        (self.0)(i)
    }
}

// inspect

pub struct FnIns<I, F: Fn(&I) + Copy>(F, PhantomData<I>);

impl<I, F: Fn(&I) + Copy> Clone for FnIns<I, F> {
    fn clone(&self) -> Self {
        Self::new(self.0)
    }
}

impl<I, F: Fn(&I) + Copy> Copy for FnIns<I, F> {}

impl<I, F: Fn(&I) + Copy> FnIns<I, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<I, F: Fn(&I) + Copy> Map for FnIns<I, F> {
    type I = I;

    type O = I;

    #[inline(always)]
    fn map(&self, i: Self::I) -> Self::O {
        (self.0)(&i);
        i
    }
}

// cloned

pub struct FnCloned<'a, I: Clone>(PhantomData<&'a I>);

impl<'a, I: Clone> Clone for FnCloned<'a, I> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<'a, I: Clone> Copy for FnCloned<'a, I> {}

impl<'a, I: Clone> FnCloned<'a, I> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<'a, I: Clone> Map for FnCloned<'a, I> {
    type I = &'a I;

    type O = I;

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

impl<'a, I: Copy> FnCopied<'a, I> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<'a, I: Copy> Map for FnCopied<'a, I> {
    type I = &'a I;

    type O = I;

    fn map(&self, i: Self::I) -> Self::O {
        *i
    }
}
