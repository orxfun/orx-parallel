use crate::xap::fun::filter_map::fn_trait::FilterMap;
use core::marker::PhantomData;

// filter_map

pub struct FnFilMap<I, O, F: Fn(I) -> Option<O>>(F, PhantomData<(I, O)>);

impl<I, O, F: Fn(I) -> Option<O>> FnFilMap<I, O, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<I, O, F: Fn(I) -> Option<O>> FilterMap for FnFilMap<I, O, F> {
    type I = I;

    type O = O;

    #[inline(always)]
    fn filter_map(&self, i: Self::I) -> Option<Self::O> {
        (self.0)(i)
    }
}

// filter

pub struct FnFil2<I, F: Fn(&I) -> bool>(F, PhantomData<I>);

impl<I, F: Fn(&I) -> bool> FnFil2<I, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<I, F: Fn(&I) -> bool> FilterMap for FnFil2<I, F> {
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
