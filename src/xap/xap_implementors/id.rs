use crate::xap::FlaMap;
use crate::xap::count::One;
use crate::xap::fun::filter_map::{FnFil2, FnFilMap};
use crate::xap::fun::flat_map::FnFlatMap;
use crate::xap::fun::map::{FnCloned, FnCopied, FnIns, FnMap};
use crate::xap::xap_implementors::fil_map::FilMap;
use crate::xap::xap_implementors::m2::M2;
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
        = M2<Self, FnMap<Self::O, Q, H>>
    where
        H: Fn(Self::O) -> Q;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q,
    {
        M2::new(self, FnMap::new(h))
    }

    type Inspect<H>
        = M2<Self, FnIns<Self::O, H>>
    where
        H: Fn(&Self::O);

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O),
    {
        M2::new(self, FnIns::new(h))
    }

    type Filter<H>
        = FilMap<Self, FnFil2<Self::O, H>>
    where
        H: Fn(&Self::O) -> bool;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool,
    {
        FilMap::new(self, FnFil2::new(h))
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
    type Cloned = M2<Self, FnCloned<'a, I>>;

    fn cloned(self) -> Self::Cloned {
        M2::new(self, FnCloned::new())
    }
}

impl<'a, I: 'a + Copy> XapCopied<'a, I> for Id<&'a I> {
    type Copied = M2<Self, FnCopied<'a, I>>;

    fn copied(self) -> Self::Copied {
        M2::new(self, FnCopied::new())
    }
}
