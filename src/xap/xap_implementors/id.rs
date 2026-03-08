use crate::xap::count::One;
use crate::xap::fun::filter::{FnFil, Fs};
use crate::xap::fun::filter_map::FnFilMap;
use crate::xap::fun::flat_map::FnFlatMap;
use crate::xap::fun::map::{FnCloned, FnCopied, FnIns, FnMap, Ms};
use crate::xap::xap_implementors::F;
use crate::xap::xap_implementors::fil_map::FilMap;
use crate::xap::xap_implementors::flat_map::FlaMap;
use crate::xap::xap_implementors::m::M;
use crate::xap::xap_trait::{Xap, XapCloned, XapCopied};
use core::marker::PhantomData;

pub struct Id<I>(PhantomData<I>);

impl<I> Id<I> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<I> Xap for Id<I> {
    type I = I;

    type O = I;

    type Count = One;

    type Values<'i>
        = [I; 1]
    where
        Self: 'i;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values<'_> {
        [i]
    }

    // transformations

    type Map<Q, H>
        = M<Self, Ms<FnMap<Self::O, Q, H>>>
    where
        H: Fn(Self::O) -> Q;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q,
    {
        M::new(self, Ms::new(FnMap::new(h)))
    }

    type Inspect<H>
        = M<Self, Ms<FnIns<Self::O, H>>>
    where
        H: Fn(&Self::O);

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O),
    {
        M::new(self, Ms::new(FnIns::new(h)))
    }

    type Filter<H>
        = F<Self, Fs<FnFil<Self::O, H>>>
    where
        H: Fn(&Self::O) -> bool;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool,
    {
        F::new(self, Fs::new(FnFil::new(h)))
    }

    type FilterMap<Q, H>
        = FilMap<Self, FnFilMap<Self::O, Q, H>>
    where
        H: Fn(Self::O) -> Option<Q>;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q>,
    {
        FilMap::new(self, FnFilMap::new(h))
    }

    type FlatMap<V, H>
        = FlaMap<Self, FnFlatMap<Self::O, V, H>>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V,
    {
        FlaMap::new(self, FnFlatMap::new(h))
    }
}

impl<'a, I: 'a + Clone> XapCloned<'a, I> for Id<&'a I> {
    type Cloned = M<Self, Ms<FnCloned<'a, I>>>;

    fn cloned(self) -> Self::Cloned {
        M::new(self, Ms::new(FnCloned::new()))
    }
}

impl<'a, I: 'a + Copy> XapCopied<'a, I> for Id<&'a I> {
    type Copied = M<Self, Ms<FnCopied<'a, I>>>;

    fn copied(self) -> Self::Copied {
        M::new(self, Ms::new(FnCopied::new()))
    }
}
