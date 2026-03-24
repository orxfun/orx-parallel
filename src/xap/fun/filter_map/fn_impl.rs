use crate::xap::fun::filter_map::fn_trait::FilterMap;
use core::marker::PhantomData;

// filter_map

pub struct FnFilMap<I, O, F: Fn(I) -> Option<O> + Copy>(F, PhantomData<(I, O)>);

impl<I, O, F: Fn(I) -> Option<O> + Copy> Clone for FnFilMap<I, O, F> {
    fn clone(&self) -> Self {
        Self::new(self.0)
    }
}

impl<I, O, F: Fn(I) -> Option<O> + Copy> Copy for FnFilMap<I, O, F> {}

impl<I, O, F: Fn(I) -> Option<O> + Copy> FnFilMap<I, O, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<I, O, F: Fn(I) -> Option<O> + Copy> FilterMap for FnFilMap<I, O, F> {
    type I = I;

    type O = O;

    #[inline(always)]
    fn filter_map(&self, i: Self::I) -> Option<Self::O> {
        (self.0)(i)
    }
}

// filter

pub struct FnFil<I, F: Fn(&I) -> bool + Copy>(F, PhantomData<I>);

impl<I, F: Fn(&I) -> bool + Copy> Clone for FnFil<I, F> {
    fn clone(&self) -> Self {
        Self::new(self.0)
    }
}

impl<I, F: Fn(&I) -> bool + Copy> Copy for FnFil<I, F> {}

impl<I, F: Fn(&I) -> bool + Copy> FnFil<I, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<I, F: Fn(&I) -> bool + Copy> FilterMap for FnFil<I, F> {
    type I = I;

    type O = I;

    #[inline(always)]
    fn filter_map(&self, i: Self::I) -> Option<Self::O> {
        match (self.0)(&i) {
            true => Some(i),
            false => None,
        }
    }
}
