use crate::xap::fun::map::fn_trait::Map;
use core::marker::PhantomData;

// map

pub struct FnMap<I, O, F: Fn(I) -> O>(F, PhantomData<(I, O)>);

impl<I, O, F: Fn(I) -> O> FnMap<I, O, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<I, O, F: Fn(I) -> O> Map for FnMap<I, O, F> {
    type I = I;

    type O = O;

    #[inline(always)]
    fn map(&self, i: Self::I) -> Self::O {
        (self.0)(i)
    }
}

// inspect

pub struct FnIns<I, F: Fn(&I)>(F, PhantomData<I>);

impl<I, F: Fn(&I)> FnIns<I, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<I, F: Fn(&I)> Map for FnIns<I, F> {
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
